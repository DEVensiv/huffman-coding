# huffman-coding
Simple command line programm to compress files using optimal huffman codes.

Generates the optimal huffman tree for a given file, compresses the file using that, and stores the tree in the resulting compressed file.

# Installation
1. You need cargo. If you don't have it installed already you can do so by following the steps from [The Cargo Book](https://doc.rust-lang.org/cargo/getting-started/installation.html)
2. Run ``cargo install --git https://github.com/devensiv/huffman-coding`` to build the crate in a temporary target directory before installing the binaries in your cargo installation's ``bin`` folder. For more information check out the [cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html) Book entry
3. Alternatively to 2. you can clone the repository move into ``huffman-coding/`` in order to run ``cargo install --path .``. This has the same effect as 2. but you are not building in a temporary target directory

## Updating
You can update your installation by re running the cargo install command. (or pulling changes before if you went with 3.)

## Usage
Encode file: ``huffman <original-file>``
Decode file: ``huffman <compressed-file> -d``

# Uninstalling
In case you want to uninstall the huffman-coding you can
1. Run ``cargo uninstall huffman`` to remove the program
2. In case you installed ``cargo`` (with rustup) just for this program and want to uninstall it too you can do so by running ``rustup self uninstall`` which will uninstall ``rustup`` and all its components.

