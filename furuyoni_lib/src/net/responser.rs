use async_trait::async_trait;

#[async_trait]
pub trait Responser<Response> {
    type Request;
    type Error: std::error::Error;

    async fn recv(&mut self) -> Result<Self::Request, Self::Error>;
    fn response(&self, message: Response) -> Result<(), Self::Error>;
}
