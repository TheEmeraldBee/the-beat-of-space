use async_trait::async_trait;

#[async_trait]
pub trait Scene {
    async fn run(&mut self) -> Option<Box<dyn Scene>>;
}