#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn save(screenshot: &image::RgbaImage, filename: &str) {
    let path = std::env::current_dir()
        .expect("Failed to resolve the screenshot output directory")
        .join(filename);
    screenshot
        .save(&path)
        .expect("Failed to save the screenshot");
    println!("Screenshot saved to {}", path.display());
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn save(screenshot: &image::RgbaImage, filename: &str) {
    use std::io::Cursor;
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

    let mut png = Cursor::new(Vec::new());
    screenshot
        .write_to(&mut png, image::ImageFormat::Png)
        .expect("Failed to encode the screenshot as PNG");
    let bytes = js_sys::Uint8Array::from(png.get_ref().as_slice());
    let parts = js_sys::Array::of1(&bytes);
    let options = BlobPropertyBag::new();
    options.set_type("image/png");
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &options)
        .expect("Failed to create the screenshot download blob");
    let url = Url::create_object_url_with_blob(&blob)
        .expect("Failed to create a URL for the screenshot download");
    let window = web_sys::window().expect("Screenshot download requires a browser window");
    let document = window
        .document()
        .expect("Screenshot download requires a browser document");
    let link: HtmlAnchorElement = document
        .create_element("a")
        .expect("Failed to create the screenshot download link")
        .dyn_into()
        .expect("An anchor element must implement HtmlAnchorElement");
    link.set_href(&url);
    link.set_download(filename);
    link.set_hidden(true);
    document
        .body()
        .expect("Screenshot download requires a document body")
        .append_child(&link)
        .expect("Failed to attach the screenshot download link");
    link.click();
    link.remove();
    // Let the browser start the download before releasing its URL.
    let revoke = Closure::once_into_js(move || {
        Url::revoke_object_url(&url).expect("Failed to release the screenshot download URL");
    });
    window
        .set_timeout_with_callback(revoke.unchecked_ref())
        .expect("Failed to schedule screenshot download URL cleanup");
    web_sys::console::log_1(&format!("Screenshot download: {filename}").into());
}
