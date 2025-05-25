use std::error::Error;
use std::ptr;
use windows::core::*;
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0};
use windows::Win32::Graphics::Direct3D11::D3D11_SDK_VERSION;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct DxgiCapture {
    // Recursos que podem ser recriados
    duplication: Option<IDXGIOutputDuplication>,
    d3d_device: Option<ID3D11Device>,
    d3d_context: Option<ID3D11DeviceContext>,
    dxgi_output5: Option<IDXGIOutput5>,
    roi_texture: Option<ID3D11Texture2D>,
    
    // Informações que persistem
    pub output_width: u32,
    pub output_height: u32,
    chosen_format: DXGI_FORMAT,
    
    // Cache do tamanho da ROI para reutilização
    roi_cached_width: u32,
    roi_cached_height: u32,
}

impl DxgiCapture {
    pub fn new() -> Result<Self> {
        let mut capture = Self {
            duplication: None,
            d3d_device: None,
            d3d_context: None,
            dxgi_output5: None,
            roi_texture: None,
            output_width: 0,
            output_height: 0,
            chosen_format: DXGI_FORMAT_B8G8R8A8_UNORM,
            roi_cached_width: 0,
            roi_cached_height: 0,
        };
        
        capture.initialize_duplication()?;
        Ok(capture)
    }
    
    /// Inicializa ou reinicializa todos os recursos DXGI
    fn initialize_duplication(&mut self) -> Result<()> {
        // Limpar recursos anteriores
        self.release_resources();
        
        // Criar o dispositivo D3D11
        let driver_types = [D3D_DRIVER_TYPE_HARDWARE];
        let mut d3d_device: Option<ID3D11Device> = None;
        let mut d3d_context: Option<ID3D11DeviceContext> = None;
        let feature_levels = [D3D_FEATURE_LEVEL_11_0];
        
        for &driver_type in &driver_types {
            unsafe {
                let hr = D3D11CreateDevice(
                    None,
                    driver_type,
                    None,
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                    Some(&feature_levels),
                    D3D11_SDK_VERSION,
                    Some(&mut d3d_device),
                    None,
                    Some(&mut d3d_context),
                );
                
                if hr.is_ok() {
                    break;
                }
            }
        }
        
        let d3d_device = d3d_device.ok_or("Falha ao criar o dispositivo D3D11")?;
        let d3d_context = d3d_context.ok_or("Falha ao criar o contexto D3D11")?;
        
        // Obter o adaptador e output
        let dxgi_device: IDXGIDevice = d3d_device.cast()?;
        let dxgi_adapter: IDXGIAdapter = unsafe { dxgi_device.GetAdapter()? };
        let dxgi_output: IDXGIOutput = unsafe { dxgi_adapter.EnumOutputs(0)? };
        let dxgi_output5: IDXGIOutput5 = dxgi_output.cast()?;
        
        // Obter dimensões
        let mut output_desc = DXGI_OUTPUT_DESC::default();
        unsafe {
            dxgi_output.GetDesc(&mut output_desc)?;
        }
        
        self.output_width = (output_desc.DesktopCoordinates.right - output_desc.DesktopCoordinates.left) as u32;
        self.output_height = (output_desc.DesktopCoordinates.bottom - output_desc.DesktopCoordinates.top) as u32;
        
        // Criar duplicação
        let supported_formats = [
            DXGI_FORMAT_B8G8R8A8_UNORM,
            DXGI_FORMAT_R8G8B8A8_UNORM,
            DXGI_FORMAT_R16G16B16A16_FLOAT
        ];
        
        let mut duplication: Option<IDXGIOutputDuplication> = None;
        
        unsafe {
            for &format in &supported_formats {
                let result = dxgi_output5.DuplicateOutput1(
                    &d3d_device,
                    0,
                    &[format],
                );
                
                if let Ok(dupl) = result {
                    duplication = Some(dupl);
                    self.chosen_format = format;
                    break;
                }
            }
        }
        
        let duplication = duplication.ok_or("Falha ao criar a duplicação de saída")?;
        
        // Armazenar recursos (sem criar textura ROI ainda)
        self.d3d_device = Some(d3d_device);
        self.d3d_context = Some(d3d_context);
        self.dxgi_output5 = Some(dxgi_output5);
        self.duplication = Some(duplication);
        
        // Resetar cache da ROI
        self.roi_cached_width = 0;
        self.roi_cached_height = 0;
        
        Ok(())
    }
    
    /// Cria ou recria a textura ROI se necessário
    fn ensure_roi_texture(&mut self, width: u32, height: u32) -> Result<()> {
        // Se já temos uma textura com o tamanho correto, reutilizar
        if self.roi_texture.is_some() && 
           self.roi_cached_width == width && 
           self.roi_cached_height == height {
            return Ok(());
        }
        
        // Limpar textura antiga
        self.roi_texture = None;
        
        // Criar nova textura com o tamanho exato necessário
        let roi_texture_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: self.chosen_format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: D3D11_BIND_FLAG(0),
            CPUAccessFlags: D3D11_CPU_ACCESS_FLAG(D3D11_CPU_ACCESS_READ.0),
            MiscFlags: D3D11_RESOURCE_MISC_FLAG(0),
        };
        
        let mut roi_texture: Option<ID3D11Texture2D> = None;
        unsafe {
            if let Some(device) = &self.d3d_device {
                device.CreateTexture2D(
                    &roi_texture_desc,
                    None,
                    Some(&mut roi_texture),
                )?;
            }
        }
        
        self.roi_texture = roi_texture;
        self.roi_cached_width = width;
        self.roi_cached_height = height;
        
        Ok(())
    }
    
    /// Libera todos os recursos DXGI
    fn release_resources(&mut self) {
        self.duplication = None;
        self.roi_texture = None;
        self.dxgi_output5 = None;
        self.d3d_context = None;
        self.d3d_device = None;
        self.roi_cached_width = 0;
        self.roi_cached_height = 0;
    }
    
    pub fn capture_region(&mut self, left: u32, top: u32, width: u32, height: u32) -> Result<Vec<u8>> {
        if left + width > self.output_width || top + height > self.output_height {
            return Err("Região solicitada fora dos limites da tela".into());
        }
        
        // Verificar se temos uma duplicação válida
        if self.duplication.is_none() {
            self.initialize_duplication()?;
        }
        
        // Garantir que temos uma textura ROI do tamanho correto
        self.ensure_roi_texture(width, height)?;
        
        let mut frame_resource: Option<IDXGIResource> = None;
        let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
        
        unsafe {
            let duplication = self.duplication.as_ref().unwrap();
            let result = duplication.AcquireNextFrame(
                0,
                &mut frame_info,
                &mut frame_resource,
            );
            
            if let Err(err) = result {
                let error_code = err.code();
                
                // Erros que requerem reinicialização
                if error_code == DXGI_ERROR_ACCESS_LOST || 
                   error_code == DXGI_ERROR_DEVICE_REMOVED || 
                   error_code == DXGI_ERROR_DEVICE_RESET ||
                   error_code == DXGI_ERROR_SESSION_DISCONNECTED {
                    
                    // Tentar reinicializar
                    match self.initialize_duplication() {
                        Ok(_) => {
                            // Após reinicialização, precisamos recriar a textura ROI
                            self.ensure_roi_texture(width, height)?;
                        }
                        Err(e) => {
                            eprintln!("Falha ao reinicializar: {}", e);
                            return Err(e);
                        }
                    }
                }
                
                // Para outros erros, apenas retornar
                return Err(format!("Erro ao adquirir frame: {:?}", error_code).into());
            }
        }
        
        let frame_resource = match frame_resource {
            Some(resource) => resource,
            None => {
                unsafe { 
                    if let Some(dup) = &self.duplication {
                        let _ = dup.ReleaseFrame();
                    }
                }
                return Ok(vec![0u8; (height as usize) * (width as usize) * 4]);
            }
        };
        
        // Obter a textura e copiar região
        let acquired_texture: ID3D11Texture2D = frame_resource.cast()?;
        
        unsafe {
            let src_box = D3D11_BOX {
                left,
                top,
                front: 0,
                right: left + width,
                bottom: top + height,
                back: 1,
            };
            
            if let (Some(context), Some(roi_texture)) = (&self.d3d_context, &self.roi_texture) {
                context.CopySubresourceRegion(
                    roi_texture,
                    0,
                    0,
                    0,
                    0,
                    &acquired_texture,
                    0,
                    Some(&src_box),
                );
            }
        }
        
        // Liberar o frame
        unsafe {
            if let Some(dup) = &self.duplication {
                let _ = dup.ReleaseFrame();
            }
        }
        
        // Mapear e copiar dados
        let mut mapped_resource = D3D11_MAPPED_SUBRESOURCE::default();
        unsafe {
            if let (Some(context), Some(roi_texture)) = (&self.d3d_context, &self.roi_texture) {
                context.Map(
                    roi_texture,
                    0,
                    D3D11_MAP_READ,
                    0,
                    Some(&mut mapped_resource),
                )?;
            }
        }
        
        let row_pitch = mapped_resource.RowPitch;
        let mut buffer = vec![0u8; (height as usize) * (width as usize) * 4];
        
        unsafe {
            let src_ptr = mapped_resource.pData as *const u8;
            
            for y in 0..height as usize {
                let src_row = src_ptr.add(y * row_pitch as usize);
                let dst_row = buffer.as_mut_ptr().add(y * width as usize * 4);
                ptr::copy_nonoverlapping(src_row, dst_row, width as usize * 4);
            }
            
            if let Some(context) = &self.d3d_context {
                if let Some(roi_texture) = &self.roi_texture {
                    context.Unmap(roi_texture, 0);
                }
            }
        }
        
        Ok(buffer)
    }
}

impl Drop for DxgiCapture {
    fn drop(&mut self) {
        self.release_resources();
    }
}