# Dxgiscreengrab

Uma biblioteca Rust para captura eficiente de regiões da tela no Windows usando DirectX Graphics Infrastructure (DXGI) e Direct3D 11.

## 📋 Visão Geral

O `DxgiCapture` é uma implementação otimizada para capturar regiões específicas da tela com alta performance, utilizando a API nativa do Windows. A biblioteca oferece cache inteligente de recursos e recuperação automática de erros.

## ✨ Funcionalidades

- **Captura de Região**: Captura áreas específicas da tela por coordenadas
- **Cache Inteligente**: Reutiliza texturas quando as dimensões não mudam
- **Recuperação Automática**: Reinicializa recursos automaticamente em caso de erros
- **Múltiplos Formatos**: Suporte para diferentes formatos de pixel (BGRA, RGBA, Float16)
- **Performance Otimizada**: Usa Direct3D 11 e DXGI para máxima eficiência

## 🔧 Dependências

Adicione ao seu `Cargo.toml`:

```toml
[dependencies]
windows = { version = "0.51", features = [
    "Win32_Graphics_Direct3D",
    "Win32_Graphics_Direct3D11", 
    "Win32_Graphics_Dxgi",
    "Win32_Graphics_Dxgi_Common"
]}
```

## 🚀 Uso Básico

```rust
use dxgi_capture::DxgiCapture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Criar instância do capturador
    let mut capture = DxgiCapture::new()?;
    
    // Capturar região específica (left, top, width, height)
    let pixel_data = capture.capture_region(100, 100, 800, 600)?;
    
    // Os dados estão em formato BGRA (4 bytes por pixel)
    println!("Capturados {} bytes", pixel_data.len());
    
    Ok(())
}
```

## 📖 API Detalhada

### `DxgiCapture::new()`

Cria uma nova instância do capturador, inicializando todos os recursos necessários.

**Retorna**: `Result<DxgiCapture, Box<dyn Error>>`

### `capture_region(left, top, width, height)`

Captura uma região específica da tela.

**Parâmetros**:
- `left: u32` - Coordenada X do canto superior esquerdo
- `top: u32` - Coordenada Y do canto superior esquerdo  
- `width: u32` - Largura da região
- `height: u32` - Altura da região

**Retorna**: `Result<Vec<u8>, Box<dyn Error>>`

Os dados retornados estão no formato BGRA com 4 bytes por pixel.

### Propriedades Públicas

- `output_width: u32` - Largura total da tela
- `output_height: u32` - Altura total da tela

## 🏗️ Arquitetura Interna

### Gerenciamento de Recursos

A biblioteca implementa um sistema sofisticado de cache e recuperação:

- **Cache de Texturas**: Reutiliza texturas D3D11 quando as dimensões da região não mudam
- **Recuperação Automática**: Detecta e recupera automaticamente de erros como:
  - `DXGI_ERROR_ACCESS_LOST`
  - `DXGI_ERROR_DEVICE_REMOVED`
  - `DXGI_ERROR_DEVICE_RESET`
  - `DXGI_ERROR_SESSION_DISCONNECTED`

### Formatos Suportados

A biblioteca tenta os seguintes formatos em ordem de preferência:

1. `DXGI_FORMAT_B8G8R8A8_UNORM` (BGRA 8-bit)
2. `DXGI_FORMAT_R8G8B8A8_UNORM` (RGBA 8-bit)  
3. `DXGI_FORMAT_R16G16B16A16_FLOAT` (RGBA 16-bit float)

## 🔍 Exemplo Avançado

```rust
use dxgi_capture::DxgiCapture;

fn capture_screenshot_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut capture = DxgiCapture::new()?;
    
    // Obter dimensões da tela
    let screen_width = capture.output_width;
    let screen_height = capture.output_height;
    
    println!("Resolução da tela: {}x{}", screen_width, screen_height);
    
    // Capturar tela inteira
    let pixels = capture.capture_region(0, 0, screen_width, screen_height)?;
    
    // Converter para imagem (exemplo usando image crate)
    // let img = image::RgbaImage::from_raw(screen_width, screen_height, pixels)
    //     .ok_or("Falha ao criar imagem")?;
    // img.save("screenshot.png")?;
    
    Ok(())
}

fn capture_window_region() -> Result<(), Box<dyn std::error::Error>> {
    let mut capture = DxgiCapture::new()?;
    
    // Capturar região específica (ex: janela de aplicativo)
    let x = 200;
    let y = 150; 
    let width = 1024;
    let height = 768;
    
    let pixels = capture.capture_region(x, y, width, height)?;
    
    // Processar pixels...
    
    Ok(())
}
```

### Requisitos do Sistema

- **Windows 8 ou superior**: Requer DXGI 1.2+
- **DirectX 11**: Hardware compatível necessário
- **Drivers atualizados**: Drivers de vídeo atualizados recomendados

### Tratamento de Erros

A biblioteca trata automaticamente a maioria dos erros comuns:

- Reinicialização automática quando o dispositivo é perdido
- Validação de limites para regiões de captura
- Fallback para diferentes formatos de pixel

## 🤝 Contribuição

Contribuições são bem-vindas! Por favor:

1. Faça um fork do projeto
2. Crie uma branch para sua feature
3. Commit suas mudanças
4. Abra um Pull Request

## 📞 Suporte

Para problemas ou dúvidas:

- Abra uma issue no repositório
- Verifique a documentação do Windows sobre DXGI
- Confirme que seu sistema atende aos requisitos mínimos
