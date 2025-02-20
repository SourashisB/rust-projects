use tch::{nn, Device, Tensor, nn::Module, vision::image};
use image::{DynamicImage, GenericImageView};

fn main() {
    // Load the pre-trained model
    let model_path = "mnist_model.pt";
    let device = Device::cuda_if_available();
    let vs = nn::VarStore::new(device); 
    let model = MnistModel::new(&vs.root());

    vs.load(model_path).expect("Failed to load model");

    // Load and preprocess the image
    let image_path = "digit.png"; // Ensure this image is a 28x28 grayscale digit
    let img = image::open(image_path).expect("Failed to open image");
    let tensor = preprocess_image(img);

    // Run inference
    let output = model.forward(&tensor);
    let prediction = output.argmax(1, false);
    
    println!("Predicted digit: {}", prediction.int64_value(&[]));
}

/// Define the neural network structure
#[derive(Debug)]
struct MnistModel {
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    fc1: nn::Linear,
    fc2: nn::Linear,
}

impl MnistModel {
    fn new(vs: &nn::Path) -> Self {
        let conv1 = nn::conv2d(vs, 1, 32, 5, Default::default());
        let conv2 = nn::conv2d(vs, 32, 64, 5, Default::default());
        let fc1 = nn::linear(vs, 1024, 128, Default::default());
        let fc2 = nn::linear(vs, 128, 10, Default::default());

        MnistModel { conv1, conv2, fc1, fc2 }
    }
}

impl nn::Module for MnistModel {
    fn forward(&self, xs: &Tensor) -> Tensor {
        xs.view([-1, 1, 28, 28])
            .apply(&self.conv1)
            .relu()
            .max_pool2d_default(2)
            .apply(&self.conv2)
            .relu()
            .max_pool2d_default(2)
            .view([-1, 1024])
            .apply(&self.fc1)
            .relu()
            .apply(&self.fc2)
            .softmax(-1, tch::Kind::Float)
    }
}

/// Preprocess the image for the model
fn preprocess_image(img: DynamicImage) -> Tensor {
    let grayscale = img.to_luma8();
    let resized = image::imageops::resize(&grayscale, 28, 28, image::imageops::FilterType::Gaussian);
    
    let img_vec: Vec<f32> = resized.pixels().map(|p| p.0[0] as f32 / 255.0).collect();
    
    Tensor::of_slice(&img_vec)
        .view([1, 1, 28, 28])
        .to_device(Device::cuda_if_available()) // Move tensor to GPU if available
}