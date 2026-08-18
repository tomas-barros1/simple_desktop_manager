# Packaging & AUR Contrib Files

Este diretório contém os arquivos necessários para empacotar o **`simple_menu_manager`** para o **Arch Linux User Repository (AUR)** e outras distribuições.

---

## 📦 Conteúdo do Diretório

- **`PKGBUILD`**: Arquivo de compilação oficial do Arch Linux a partir do código-fonte (para o pacote `simple-menu-manager`).
- **`PKGBUILD.bin`**: Alternativa para empacotamento do binário pré-compilado das Releases do GitHub (`simple-menu-manager-bin`).
- **`.SRCINFO`**: Metadados gerados para o AUR.
- **`update-aur.sh`**: Script para atualizar automaticamente a versão, calcular o SHA256 do tarball e atualizar o `PKGBUILD` e `.SRCINFO`.
- **`calc-sha256.sh`**: Utilitário para calcular e exibir o checksum SHA256 de qualquer tag, release ou arquivo local.

---

## 🚀 Como Publicar / Atualizar no AUR

### 1. Atualizar Versão e Checksum
Ao lançar uma nova versão (ex: `v0.1.2`):

```bash
chmod +x contrib/update-aur.sh contrib/calc-sha256.sh
./contrib/update-aur.sh 0.1.2
```

O script irá:
1. Buscar o tarball oficial da tag no GitHub.
2. Calcular o hash SHA256 real.
3. Atualizar o `PKGBUILD` com a nova versão e o novo hash.
4. Regenerar o arquivo `.SRCINFO`.

### 2. Testar o Pacote Localmente
No Arch Linux (ou Manjaro/EndeavourOS):

```bash
cd contrib
makepkg -si
```

### 3. Enviar para o AUR
Se você for o mantenedor no AUR:

```bash
# Clone o repositório AUR (apenas na primeira vez)
git clone ssh://aur@aur.archlinux.org/simple-menu-manager.git /tmp/aur-simple-menu-manager

# Copie os arquivos atualizados
cp contrib/PKGBUILD contrib/.SRCINFO /tmp/aur-simple-menu-manager/

# Envie para o AUR
cd /tmp/aur-simple-menu-manager
git add PKGBUILD .SRCINFO
git commit -m "Update to v0.1.2"
git push
```

---

## 🔒 Cálculo Manual de SHA256

Se preferir calcular o SHA256 manualmente:

```bash
# Para uma tag do Git
./contrib/calc-sha256.sh --tag v0.1.1

# Para um binário de Release
./contrib/calc-sha256.sh --release 0.1.1

# Para um arquivo local
./contrib/calc-sha256.sh target/release/simple_menu_manager
```
