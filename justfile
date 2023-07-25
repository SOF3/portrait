set shell := ["bash", "-O", "globstar", "-c"]

fmt:
    rustfmt +nightly ./**/*.rs
