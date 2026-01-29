# CodeQL Setup Instructions

## ✅ Configuração Atual

O repositório usa **Default Setup** do GitHub para análise CodeQL.

O CodeQL é gerenciado automaticamente pelo GitHub através da interface:

-   Configurado em **Settings** → **Code security and analysis**
-   Executa automaticamente em pushes e pull requests
-   Não requer configuração manual no workflow

## ⚙️ Como Funciona o Default Setup

O Default Setup do GitHub:

-   ✅ É gerenciado automaticamente pelo GitHub
-   ✅ Executa análise CodeQL em cada push e pull request
-   ✅ Detecta automaticamente a linguagem (Rust)
-   ✅ Usa configurações otimizadas para Rust
-   ✅ Não requer configuração manual no workflow
-   ✅ Gera resultados automaticamente na aba "Security"

## 📋 Verificação

Para verificar se o CodeQL está ativo:

1.  Acesse o repositório: [GitHub](https://github.com/marlonsc/mcb)
2.  Vá para a aba **Security** (no topo do repositório)
3.  Clique em **Code scanning** no menu lateral
4.  Você deve ver os resultados das análises CodeQL

## 🔧 Habilitar/Desabilitar Default Setup

Se precisar gerenciar o CodeQL:

1.  Acesse **Settings** → **Code security and analysis**
2.  Encontre **CodeQL analysis**
3.  Use o menu (•••) para:

-   **Edit**: Modificar configurações
-   **Disable CodeQL**: Desabilitar temporariamente
-   **Enable CodeQL**: Reativar se desabilitado

## ✅ Vantagens do Default Setup

-   **Simplicidade**: Configuração automática, sem manutenção
-   **Otimizado**: GitHub usa configurações otimizadas para Rust
-   **Confiável**: Mantido e atualizado pelo GitHub
-   **Sem conflitos**: Não há conflito entre Default e Advanced Setup
