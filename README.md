# bmp_rs

A bitmap file decoder.

## Development Status

This create is in its early stages of development and not all features are fully implemented yet. The public API is still subject to change and documentation is very rare. The major features for this crate are as follows (checked ones are done but still subject to change):

- [ ] Microsoft BMP Version 1 Support (If there is interest)
- [x] Microsoft BMP Version 2 support
- [ ] Microsoft BMP Version 3 support
- [ ] Microsoft BMP Version 4 support
- [ ] Microsoft BMP Version 5 Support
- [ ] IBM OS/2 2.x BMP Support
- [ ] Adobe Photoshop BMP Support (At least the "documented" version)
- [ ] OS/2 Bitmap Array support
- [ ] OS/2 Color Icon support
- [ ] OS/2 Color Pointer support
- [ ] OS/2 Struct Icon support
- [ ] OS/2 Pointer support
- [ ] A general bitmap Version 5 writer (writing in other formats makes no sense for now)
- [ ] Tests (unit and integration)
- [ ] Documentation
- [ ] Examples

## Example

```rust
use std::fs::File;
use bmp_rs::{
    Result,
    BMPDecorder,
};

struct ImageDecoder {
    // your builder type that is able to construct an image
}


impl BMPDecoder for ImageDecoder {
    type TResult = MyImageType; // Your image type

    fn set_size( &mut self, width: u32, height: u32 ) {
        // Set image size
    }

    fn set_pixel( &mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8 ) {
        // Set a specific pixel within that image to the given color
    }

    fn build( &mut self ) -> Result<Self::TResult> {
        // Build and return your final image
    }
}

fn main() {
    let mut file = File::open( "image.bmp" ).unwrap();
    let image = bmp_rs::decode( &mut file, YourImageDecoderInstance );
    // Do something with your image
}
```

## License

See [LICENSE](LICENSE) file.
