# bmp_rs

A bitmap file decoder for Microsoft *bmp* files.

## Development Status

The following is a rough list of things that are already supported or will be in the future:

- [x] 1-bit bitmap
- [x] 4-bit uncompressed bitmap
- [ ] 4-bit compressed bitmap
- [x] 8-bit uncompressed bitmap
- [ ] 8-bit compressed bitmap
- [x] 16-bit bitmap
- [x] 24-bit bitmap
- [x] 32-bit bitmap
- [ ] Microsoft BMP Version 1 header
- [x] Microsoft BMP Version 2 header
- [x] Microsoft BMP Version 3 header
- [ ] Microsoft BMP Version 4 header
- [ ] Microsoft BMP Version 5 header
- [x] IBM OS/2 1.x BMP header (32k x 32k limit)
- [ ] IBM OS/2 2.x BMP header
- [ ] OS/2 Bitmap Array type
- [ ] OS/2 Color Icon type
- [ ] OS/2 Color Pointer type
- [ ] OS/2 Struct Icon type
- [ ] OS/2 Pointer type
- [ ] Bitmap Encoding
- [ ] Test suite
- [ ] Documentation
- [ ] Examples

## Example

```rust
use std::fs::File;
use bmp_rs::{
    Result,
    Decoder,
};

struct ImageDecoder {
    // Your builder type that is able to construct an image
}

struct Image {
    // Your image type that represents a bitmap
}

impl Decoder for ImageDecoder {
    type TResult = Image; // Your image type

    fn set_size( &mut self, width: u32, height: u32 ) {
        // Set image size
    }

    fn set_pixel( &mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8 ) {
        // Set a specific pixel within that image to the given color
    }

    fn build( &mut self ) -> Result<Self::TResult> {
        // Build and return your final image
        Ok ( Image { } )
    }
}

fn main() {
    let mut file = File::open( "image.bmp" ).unwrap();
    let image = bmp_rs::decode( &mut file, ImageDecoder { } );
    // Do something with your image
}
```

## License

See [LICENSE](LICENSE) file.
