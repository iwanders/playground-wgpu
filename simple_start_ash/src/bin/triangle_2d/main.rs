use log::*;
use simple_start::State;
use zerocopy_derive::{Immutable, IntoBytes};

// https://sotrh.github.io/learn-wgpu/beginner/tutorial4-buffer/

struct LocalState(pub State);

impl std::ops::Deref for LocalState {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.0, 0.0, 0.0],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [1.0, 0.0, 0.0],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.0, 1.0, 0.0],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.0, 0.0, 1.0],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.0, 0.0, 0.0],
    }, // E
];
const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

impl LocalState {
    pub async fn draw(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

async fn async_main() -> std::result::Result<(), anyhow::Error> {
    let mut state = LocalState(State::new(256, 256).await?);
    state.draw().await?;
    state.save("/tmp/triangle_2d.png").await?;

    Ok(())
}

pub fn main() -> std::result::Result<(), anyhow::Error> {
    env_logger::builder()
        .is_test(false)
        .filter_level(log::LevelFilter::Info)
        // .filter_level(log::LevelFilter::max())
        .try_init()?;
    pollster::block_on(async_main())?;
    println!("Hello, world! ");
    Ok(())
}
