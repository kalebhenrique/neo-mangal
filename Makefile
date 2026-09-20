.PHONY: all build install link clean test check

# Default target
all: build

# Build optimized release binary in target/release/neo-mangal
build:
	cargo build --release

# Build and create symlinks in ~/.local/bin (auto-updates on every cargo build --release)
link: build
	@mkdir -p $(HOME)/.local/bin
	@ln -sf $(CURDIR)/target/release/neo-mangal $(HOME)/.local/bin/neo-mangal
	@ln -sf $(CURDIR)/target/release/neo-mangal $(HOME)/.local/bin/nmangal
	@ln -sf $(CURDIR)/target/release/neo-mangal $(HOME)/.local/bin/neomangal
	@echo "✨ Symlinks created in ~/.local/bin:"
	@echo "   - neo-mangal -> $(CURDIR)/target/release/neo-mangal"
	@echo "   - nmangal    -> $(CURDIR)/target/release/neo-mangal"
	@echo "   - neomangal  -> $(CURDIR)/target/release/neo-mangal"
	@echo "🎉 You can now run 'nmangal' or 'neomangal' from any directory!"

# Copy physical binary to ~/.local/bin
install: build
	@mkdir -p $(HOME)/.local/bin
	@cp target/release/neo-mangal $(HOME)/.local/bin/neo-mangal
	@ln -sf $(HOME)/.local/bin/neo-mangal $(HOME)/.local/bin/nmangal
	@ln -sf $(HOME)/.local/bin/neo-mangal $(HOME)/.local/bin/neomangal
	@echo "✨ Binary copied to ~/.local/bin and 'nmangal' / 'neomangal' shortcuts created!"

# Run test suite
test:
	cargo test

# Quick compilation check
check:
	cargo check

clean:
	cargo clean
