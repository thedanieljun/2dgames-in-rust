use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct Story {
    pub story_name: String,
    pub scenes: Vec<NamedScene>,
}

#[derive(Serialize, Deserialize)]
pub struct NamedScene {
    pub scene_name: String,
    pub scene: Scene,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Scene {
    pub name: String,
    pub message: String,
    pub responses: Vec<Response>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Response {
    pub response: String,
    pub goto: String,
}

pub fn parse_story() -> Result<Story> {
    let data = include_str!("script.json");
    let story: Story = serde_json::from_str(data)?;

    Ok(story)
}
