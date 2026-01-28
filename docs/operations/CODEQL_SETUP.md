# CodeQL Setup Instructions

## ⚠️ Warning Atual

Se você está vendo este warning no PR:
```
1 configuration not found
Warning: Code scanning cannot determine the alerts introduced by this pull request, 
because 1 configuration present on refs/heads/main was not found: Default setup
```

Isso acontece porque o repositório tem **ambos** os setups habilitados:
- **Default setup** (configurado na UI do GitHub na branch `main`)
- **Advanced setup** (workflow manual no PR)

## ✅ Solução: Desabilitar Default Setup

Siga estes passos **exatos** para resolver:

### Passo 1: Acessar as Configurações
1. Abra o repositório no GitHub: https://github.com/marlonsc/mcb
2. Clique na aba **Settings** (no topo do repositório)
3. No menu lateral esquerdo, clique em **Code security and analysis**
   - Se não encontrar, procure por **Security** → **Code scanning**

### Passo 2: Localizar CodeQL Analysis
1. Na seção **Code security and analysis**, procure por **CodeQL analysis**
2. Você verá algo como:
   ```
   CodeQL analysis
   [Status: Enabled] [•••]
   ```

### Passo 3: Desabilitar Default Setup
1. Clique no menu de três pontos (**•••**) ao lado de "CodeQL analysis"
2. Você verá opções como:
   - **Edit**
   - **Switch to advanced**
   - **Disable CodeQL**
3. **Escolha uma das opções:**
   - **Opção A (Recomendada)**: Clique em **"Switch to advanced"**
     - Isso migra para Advanced Setup mantendo a análise ativa
   - **Opção B**: Clique em **"Disable CodeQL"** e depois reative manualmente
4. Confirme a ação quando solicitado

### Passo 4: Verificar
1. Após desabilitar, verifique que:
   - O status de "CodeQL analysis" mostra apenas o workflow manual
   - Não há mais "Default setup" ativo
2. Em um novo PR, o warning não deve mais aparecer

## 📋 Instruções Visuais (Passo a Passo)

```
GitHub Repository
  └─ Settings (aba no topo)
      └─ Code security and analysis (menu lateral)
          └─ CodeQL analysis
              └─ [•••] (menu de três pontos)
                  └─ "Switch to advanced" ou "Disable CodeQL"
```

## 🔍 Verificação Pós-Configuração

Após desabilitar o Default Setup:

1. **Verifique o workflow**: O CodeQL deve rodar apenas via `.github/workflows/ci.yml`
2. **Teste em um novo PR**: O warning não deve mais aparecer
3. **Confirme os resultados**: Os resultados do CodeQL devem aparecer normalmente

## ⚙️ Configuração Atual (Advanced Setup)

O workflow atual (`.github/workflows/ci.yml`) está configurado para:
- ✅ Rodar em cada push e pull request
- ✅ Analisar código Rust
- ✅ Usar queries de segurança e qualidade (`+security-and-quality`)
- ✅ Ter permissões corretas (`security-events: write`)
- ✅ Usar autobuild para Rust (modo `none`)

## ⚠️ Importante

**Este warning NÃO bloqueia merges de PRs!**

- O CodeQL está funcionando corretamente
- A análise está sendo executada
- O warning é apenas informativo sobre configuração
- Você pode fazer merge do PR normalmente

## 🆘 Troubleshooting

### Se não encontrar "Code security and analysis":
- Verifique se você tem permissões de administrador no repositório
- Alguns repositórios podem ter o menu em **Security** → **Code scanning**

### Se "Switch to advanced" não aparecer:
- O repositório pode já estar usando Advanced Setup
- Nesse caso, o warning pode ser resolvido apenas fazendo merge do PR

### Se o warning persistir após desabilitar:
- Aguarde alguns minutos para o GitHub processar a mudança
- Crie um novo PR para testar
- Verifique se o workflow `.github/workflows/ci.yml` está na branch `main`
