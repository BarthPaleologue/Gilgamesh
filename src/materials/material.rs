use crate::materials::blinn_phong::BlinnPhongMaterial;
use crate::materials::pbr::PbrMaterial;

/// A surface description that can be shared by meshes and interpreted by each renderer.
/// Configure it before sharing, or replace it with a new description when editing.
#[derive(Clone)]
pub enum Material {
    Pbr(PbrMaterial),
    BlinnPhong(BlinnPhongMaterial),
}

impl From<PbrMaterial> for Material {
    fn from(material: PbrMaterial) -> Self {
        Self::Pbr(material)
    }
}

impl From<BlinnPhongMaterial> for Material {
    fn from(material: BlinnPhongMaterial) -> Self {
        Self::BlinnPhong(material)
    }
}
