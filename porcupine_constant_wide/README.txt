A quick conveinence proc macro to generate a UTF-16 WStr. Given:

`constant_text!("Hello world!")`

It should produce:

`&WStr::from_bytes_unchecked(&[72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33, 0])`
