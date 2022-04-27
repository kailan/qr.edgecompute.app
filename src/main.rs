use fastly::http::{Method, StatusCode};
use fastly::{mime, Error, Request, Response};
use image::png::PngEncoder;
use image::{ColorType, Luma};
use qrcode::QrCode;
use serde::Deserialize;

#[derive(Deserialize)]
struct CodeParams {
    url: Option<String>,
}

fn handle_request(req: Request) -> Result<(), Error> {
    match (req.get_method(), req.get_path()) {
        (&Method::GET, "/qr.png") => {
            // Fetch target URL from query string
            let params: CodeParams = req.get_query()?;

            if let Some(url) = params.url {
                // Limit the content length
                if url.len() > 128 {
                    Response::from_status(StatusCode::BAD_REQUEST)
                        .with_body_text_plain("URL too long")
                        .stream_to_client();
                    return Ok(());
                }

                // Create a QR code
                let code = QrCode::new(url).unwrap();

                // Render the QR code as a bitmap
                let image = code
                    .render::<Luma<u8>>()
                    .min_dimensions(512, 512)
                    .quiet_zone(false)
                    .build();

                // Send the response headers to the client and get a writable stream
                let resp = Response::from_status(StatusCode::OK).with_content_type(mime::IMAGE_PNG);
                let body = resp.stream_to_client();

                // Encode the rendered image as a PNG and stream it to the client
                let encoder = PngEncoder::new(body);
                let (width, height) = image.dimensions();
                encoder.encode(&image.into_raw(), width, height, ColorType::L8)?;
            } else {
                Response::from_status(StatusCode::BAD_REQUEST)
                    .with_body_text_plain("No URL provided")
                    .send_to_client();
            }
        }
        _ => {
            Response::from_status(StatusCode::NOT_FOUND).send_to_client();
        }
    }

    Ok(())
}

// Cannot use #[fastly::main] here because we need to send a streaming body
fn main() {
    let req = Request::from_client();
    if let Err(err) = handle_request(req) {
        Response::from_status(StatusCode::INTERNAL_SERVER_ERROR).send_to_client();
        panic!("{:?}", err);
    }
}
