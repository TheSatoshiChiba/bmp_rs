# bmp_rs

A bitmap file decoder for Microsoft *bmp* files.

## Development Status

The following is a rough list of things that are already supported or will be in the future:

- [ ] Microsoft BMP Version 1 Support
- [x] Microsoft BMP Version 2 support
- [x] Basic Microsoft BMP Version 3 support
- [ ] Encoding support for BMP Version 3
- [ ] 16-/32-bit image support for BMP Version 3
- [ ] Microsoft BMP Version 4 support
- [ ] Microsoft BMP Version 5 Support
- [ ] IBM OS/2 2.x BMP Support
- [ ] OS/2 Bitmap Array support
- [ ] OS/2 Color Icon support
- [ ] OS/2 Color Pointer support
- [ ] OS/2 Struct Icon support
- [ ] OS/2 Pointer support
- [ ] Bitmap Encoding (A simple writer)
- [ ] Tests (Only internal manual tests for now)
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
