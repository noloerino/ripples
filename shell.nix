with import <nixpkgs> { }; 

runCommand "dummy" {
    buildInputs = [ cargo gcc nodejs rustup wasm-pack ];
} ""
