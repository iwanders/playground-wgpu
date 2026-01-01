use anyhow::Context as _;
use image::ImageReader;

#[test]
fn main() -> Result<(), anyhow::Error> {
    // Run the damaged helmet binary.
    let status = std::process::Command::new("cargo")
        .arg("r")
        .arg("--release")
        .arg("--bin")
        .arg("damaged_helmet_pbr")
        .env("RENDER_ENGINE_NON_INTERACTIVE", "1")
        .status()?;
    assert_eq!(status.code(), Some(0));

    let output_path = "/tmp/damaged_helmet_pbr.png";
    // Read the image.
    let image = ImageReader::open(output_path)
        .with_context(|| format!("could not read file {output_path}"))?
        .with_guessed_format()?
        .decode()?;
    let rgba8 = image.to_rgba8();

    let expected_values = [
        // Back side of helmet.
        ((263u32, 158u32), [67u8, 74, 78, 255]),
        // Specular metal at the front.
        ((626, 592), [248, 248, 245, 255]),
        // Emissive
        ((694, 427), [55, 182, 203, 255]),
        // Some rust.
        ((405, 206), [47, 24, 29, 255]),
    ];

    for ((x, y), expected) in expected_values.iter() {
        let value = rgba8.get_pixel(*x, *y);
        assert_eq!(
            value.0, *expected,
            "Values at {x}, {y} did not match, expected {expected:?}"
        );
    }

    Ok(())
}
