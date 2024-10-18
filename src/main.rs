// use ndarray::{Array, CowArray};
use ort::{inputs, CPUExecutionProvider, Session};
use std::{path::PathBuf, sync::Arc};
use tokenizers::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取项目根目录路径
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 构建模型文件的完整路径
    let model_path = project_root.join("models").join("model_q4f16.onnx");
    let tokenizers_path = project_root.join("models").join("tokenizer.json");

    println!("Model path: {:?}", model_path);

    let model = Session::builder()?
        .with_execution_providers([CPUExecutionProvider::default().build()])?
        .commit_from_file(model_path)?;

    const PROMPT: &str = "The corsac fox (Vulpes corsac), also known simply as a corsac, is a medium-sized fox found in";
    let tokenizer = Tokenizer::from_file(tokenizers_path).unwrap();
    let tokens = tokenizer.encode(PROMPT, false).unwrap();
    let tokens = Arc::new(
        tokens
            .get_ids()
            .iter()
            .map(|i| *i as i64)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let input = (vec![1, tokens.len() as i64], Arc::clone(&tokens));
    println!("input: {:?}", input);
    // 运行模型
    let outputs = model.run(inputs!["input_ids" => input]?)?;
    println!("outputs: {:?}", outputs);
    // 处理输出
    // let predictions = outputs["output0"].try_extract_tensor::<f32>()?;
    // println!("Model output: {:?}", predictions);

    Ok(())
}
