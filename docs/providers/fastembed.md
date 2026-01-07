# FastEmbed Provider

O FastEmbed Provider oferece embeddings locais de alta qualidade sem dependências externas de APIs.

## Visão Geral

-   **Tipo**: Embedding Provider Local
-   **Modelo**: AllMiniLML6V2 (384 dimensões)
-   **Dependências**: Zero APIs externas
-   **Performance**: ONNX inference otimizada
-   **Download**: Modelo baixado automaticamente na primeira execução

## Configuração

### Configuração Básica

```yaml
embedding:
  provider: "fastembed"
  model: "AllMiniLML6V2"  # opcional, padrão já é este
  dimensions: 384         # opcional, padrão é 384
  max_tokens: 512         # opcional, padrão é 512
```

### Configuração Completa

```yaml
embedding:
  provider: "fastembed"
  model: "AllMiniLML6V2"
  dimensions: 384
  max_tokens: 512
```

## Características

### Vantagens

✅ **Completamente Local**: Não requer APIs externas ou chaves de API
✅ **Alta Performance**: Usa ONNX inference otimizada
✅ **Modelos de Qualidade**: Baseado em sentence-transformers
✅ **Cache Inteligente**: Modelo baixado uma vez e cacheado
✅ **Compatibilidade**: Mesmo formato de saída que OpenAI/Gemini

### Limitações

⚠️ **Download Inicial**: Requer internet para baixar o modelo na primeira execução (~23MB)
⚠️ **Memória**: Modelo carregado em memória RAM
⚠️ **CPU**: Inference em CPU (não GPU)

## Modelos Suportados

| Modelo | Dimensões | Descrição |
|--------|-----------|-----------|
| AllMiniLML6V2 | 384 | Modelo padrão, bom equilíbrio qualidade/performance |
| AllMiniLML12V2 | 384 | Versão maior com melhor qualidade |
| AllMpnetBaseV2 | 768 | Modelo de alta qualidade |
| BGE Models | Variado | Família de modelos otimizados |

## Uso Programático

```rust
use mcp_context_browser::providers::embedding::FastEmbedProvider;

// Criar provider com modelo padrão
let provider = FastEmbedProvider::new()?;

// Ou especificar modelo
let provider = FastEmbedProvider::with_model(fastembed::EmbeddingModel::AllMiniLML12V2)?;

// Gerar embedding
let embedding = provider.embed("Seu texto aqui").await?;
println!("Dimensões: {}", embedding.dimensions);
println!("Modelo: {}", embedding.model);

// Gerar embeddings em lote
let texts = vec!["Texto 1".to_string(), "Texto 2".to_string()];
let embeddings = provider.embed_batch(&texts).await?;
```

## Performance

### Benchmarks Esperados

-   **Inicialização**: ~2-5 segundos (download do modelo)
-   **Embedding único**: ~10-50ms
-   **Batch de 100**: ~100-500ms
-   **Memória**: ~100-500MB (dependendo do modelo)

### Optimization

For better performance:

-   Use batch embedding when possible
-   Cache embeddings when appropriate
-   Consider smaller models for applications with memory constraints

## Troubleshooting

### Problema: "Failed to initialize FastEmbed model"

**Solução**: Verifique conexão com internet para download do modelo na primeira execução.

### Problema: "Out of memory"

**Solução**: Use um modelo menor ou aumente a memória disponível.

### Problema: "Model download failed"

**Solução**:

1.  Verifique conexão com internet
2.  Verifique permissões de escrita no diretório de cache
3.  Tente novamente (downloads são retomáveis)

## Comparação com Outros Providers

| Provider | Local | API Key | Performance | Qualidade |
|----------|-------|---------|-------------|-----------|
| FastEmbed | ✅ | ❌ | Alta | Alta |
| Ollama | ✅ | ❌ | Média | Alta |
| OpenAI | ❌ | ✅ | Muito Alta | Muito Alta |
| Mock | ✅ | ❌ | Muito Alta | Baixa |

## Arquitetura Técnica

O FastEmbed Provider:

1.  Usa a biblioteca `fastembed` para inference ONNX
2.  Carrega modelos sentence-transformers otimizados
3.  Implementa o trait `EmbeddingProvider` do MCP Context Browser
4.  Fornece interface consistente com outros providers
5.  Gerencia cache automático de modelos

## Próximos Passos

-   Suporte a mais modelos FastEmbed
-   Configuração de execution providers (CPU/GPU)
-   Quantização automática para reduzir uso de memória
-   Cache de embeddings para textos frequentes
