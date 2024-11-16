// TODO: renderpass namespace

use ash::vk;

#[derive(Clone)]
pub struct RenderPassAttachmentOutput {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub image_layout: vk::ImageLayout,
}
