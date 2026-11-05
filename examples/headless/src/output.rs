#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn save(screenshot: &image::RgbaImage, filename: &str) {
    let path = std::env::current_dir().unwrap().join(filename);
    screenshot.save(&path).unwrap();
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
        .unwrap();
    let bytes = js_sys::Uint8Array::from(png.get_ref().as_slice());
    let parts = js_sys::Array::of1(&bytes);
    let options = BlobPropertyBag::new();
    options.set_type("image/png");
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &options).unwrap();
    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let link: HtmlAnchorElement = document.create_element("a").unwrap().dyn_into().unwrap();
    link.set_href(&url);
    link.set_download(filename);
    link.set_hidden(true);
    document.body().unwrap().append_child(&link).unwrap();
    link.click();
    link.remove();
    // Let the browser start the download before releasing its URL.
    let revoke = Closure::once_into_js(move || Url::revoke_object_url(&url).unwrap());
    window
        .set_timeout_with_callback(revoke.unchecked_ref())
        .unwrap();
    web_sys::console::log_1(&format!("Screenshot download: {filename}").into());
}
