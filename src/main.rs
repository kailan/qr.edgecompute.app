use fastly::http::{Method, StatusCode};
use fastly::{Error, mime, Request, Response};
use image::{ColorType, Luma};
use image::png::PngEncoder;
use qrcode::QrCode;
use serde::Deserialize;

#[derive(Deserialize)]
struct CodeParams {
    url: Option<String>
}

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    match (req.get_method(), req.get_path()) {
        (&Method::GET, "/qr.png") => {
            let params: CodeParams = req.get_query()?;
            if let Some(url) = params.url {
                if url.len() > 64 {
                    return Ok(Response::from_status(StatusCode::BAD_REQUEST).with_body_text_plain("URL too long"));
                }

                let code = QrCode::new(url).unwrap();
                let image = code.render::<Luma<u8>>().min_dimensions(512, 512).quiet_zone(false).build();
                let mut output = Vec::new();
                let encoder = PngEncoder::new(&mut output);
                let (width, height) = image.dimensions();
                encoder.encode(&image.into_raw(), width, height, ColorType::L8)?;
                Ok(Response::from_body(output).with_content_type(mime::IMAGE_PNG))
            } else {
                Ok(Response::from_status(StatusCode::BAD_REQUEST).with_body_text_plain("No URL provided"))
            }
        },
        _  => {
            Ok(Response::from_status(StatusCode::NOT_FOUND))
        }
    }
}
