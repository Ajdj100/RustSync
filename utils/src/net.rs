use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

pub async fn recieve(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    // read size header
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    
    //create auto sized buffer
    let len = u32::from_be_bytes(len_buf);

    // read actual payload
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;

    return Ok(buf);
}

pub async fn send(stream: &mut tokio::net::TcpStream, data: &Vec<u8>) -> Result<(), std::io::Error> {
    let len = (data.len() as u32).to_be_bytes();

    stream.write_all(&len).await?;
    stream.write_all(&data).await?;
    Ok(())
}
