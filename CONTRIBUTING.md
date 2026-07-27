# Contributing to `meteorite`

## Rules & Standards

### Code formatting

All code must be formatted with `cargo fmt`.

### Linting

Your code must pass `cargo clippy` with zero warnings.

### Branches

Your branch name **must** start with one of the following types, followed by a slash and a short, hyphen-separated description of your work:

* **`feat/`**: For adding new features and capabilities (e.g. `feat/sso-login`).
* **`fix/`**: For fixing broken code or resolving panics (e.g. `fix/prevent-overflow`).
* **`chore/`**: For maintenance, dependency updates or documentation tweaks (e.g. `chore/update-readme`).
* **`refactor/`**: For restructuring existing code without changing how it works (e.g. `refactor/cleanup-user-auth`).

### Pull Requests

To keep our Git history clean and maintainable, please ensure your Pull Request meets the following standards before submitting:

#### 1. Proper PR Title

Your PR title must be a clear, one-liner summary of your changes and **must follow the Conventional Commits specification**.

* **Good:** `feat(auth): add oauth login capability`
* **Bad:** `type/branch-name` (Do not just copy-paste your branch name as the title)

#### 2. Reasonably sized Commits

Each commit should represent a single, isolated logical change. Avoid bunching fixed typos, refactors, and new features all into one massive commit.

#### 3. Quality Commit Messages

Each commit within your branch should have a meaningful, descriptive message explaining *what* was changed and *why*.

### Commit messages

We follow the **Conventional Commits** specification.

Examples:

* `feat: add user profile`
* `fix(auth): resolve login panic`

### AI Usage

The usage of generative AI tools (such as ChatGPT, Claude, or large-scale Copilot generation) to write code or documentation for this project is strictly prohibited.

* **Allowed:** Basic IDE code-completion for syntax.
* **Prohibited:** Generating entire functions, algorithms, or copy-pasting AI-generated PR descriptions.
We value human-authored code where the contributor deeply understands the logic, edge cases, and architectural choices being made.

## How to Contribute

1. **Fork or Clone** the repository.
2. Install required tools and prerequisites:

    * [Dioxus CLI](https://dioxuslabs.com/learn/0.6/getting_started/#install-the-dioxus-cli) (required for UI development and running the desktop app)

    ### Prerequisites (Linux)

    If you are developing on Linux, you must install native C/C++ build tools and WebKitGTK development headers before building the project.

    #### Debian / Ubuntu / Linux Mint / Pop!_OS
    ```bash
    sudo apt-get update
    sudo apt-get install -y \
      build-essential \
      pkg-config \
      libglib2.0-dev \
      libgtk-3-dev \
      libjavascriptcoregtk-4.1-dev \
      libsoup-3.0-dev \
      libwebkit2gtk-4.1-dev \
      libssl-dev
    ```

    #### Fedora / RHEL / AlmaLinux
    ```bash
    sudo dnf install -y \
      @development-tools \
      pkgconf-pkg-config \
      glib2-devel \
      gtk3-devel \
      javascriptcoregtk4.1-devel \
      libsoup3-devel \
      webkit2gtk4.1-devel \
      openssl-devel
    ```

    #### Arch Linux / Manjaro
    ```bash
    sudo pacman -S --needed \
      base-devel \
      gtk3 \
      webkit2gtk-4.1 \
      libsoup3 \
      openssl
    ```
    > **Note:** On Arch Linux, development headers are bundled directly inside the main package releases.

    #### openSUSE (Tumbleweed / Leap)
    ```bash
    sudo zypper install -y -t pattern devel_basis
    sudo zypper install -y \
      pkg-config \
      glib2-devel \
      gtk3-devel \
      javascriptcoregtk-4_1-devel \
      libsoup-3_0-devel \
      webkit2gtk-4_1-devel \
      libopenssl-devel
    ```

3. **Create a new branch** from `main`:

    ```bash
    git checkout -b type/your-branch-name
    ```

3. Check out [`TODO.md`](TODO.md) or search for comments in code starting with `TODO`

**Branches that do not follow our rules will not be merged.**
