use llama_cpp_sys_2::{
    ggml_log_level, llama_backend_free, llama_backend_init, llama_batch_free, llama_batch_init,
    llama_context_default_params, llama_decode, llama_free, llama_free_model, llama_get_logits,
    llama_get_memory, llama_load_model_from_file, llama_log_set, llama_memory_clear, llama_model,
    llama_model_default_params, llama_model_get_vocab, llama_n_vocab, llama_new_context_with_model,
    llama_set_n_threads, llama_token, llama_token_eos, llama_token_to_piece, llama_tokenize,
    llama_vocab, GGML_LOG_LEVEL_ERROR,
};
use rand::prelude::*;
use std::cmp::Ordering;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::ptr;
use std::sync::Once;

const MAX_SEQ_ID: i32 = 1;
const PROMPT_CHUNK_SIZE: usize = 256;
const SAMPLING_TEMPERATURE: f32 = 0.8;
const SAMPLING_TOP_K: usize = 40;
const SAMPLING_TOP_P: f32 = 0.9;
const SAMPLING_MIN_P: f32 = 0.0;
const TOKEN_PIECE_INITIAL: usize = 64;
const TOKEN_PIECE_MAX: usize = 8192;

static LOG_INITIALIZED: Once = Once::new();

unsafe extern "C" fn llama_log_filter(level: ggml_log_level, text: *const c_char, _: *mut c_void) {
    if text.is_null() {
        return;
    }

    if level >= GGML_LOG_LEVEL_ERROR {
        if let Ok(msg) = CStr::from_ptr(text).to_str() {
            eprintln!("{msg}");
        }
    }
}

#[derive(Debug)]
pub struct LlamaSession {
    model: *mut llama_model,
    ctx: *mut llama_cpp_sys_2::llama_context,
    vocab: *const llama_vocab,
    n_ctx: i32,
}

impl LlamaSession {
    pub fn new(model_path: &Path, n_ctx: i32) -> Result<Self, String> {
        if !model_path.exists() {
            return Err(format!("Model file not found at {}", model_path.display()));
        }

        let model_path_cstr = CString::new(
            model_path
                .to_str()
                .ok_or_else(|| "Model path contains invalid UTF-8 characters".to_string())?,
        )
        .map_err(|_| "Model path contains interior null bytes".to_string())?;

        unsafe {
            LOG_INITIALIZED.call_once(|| {
                llama_log_set(Some(llama_log_filter), std::ptr::null_mut());
            });
            llama_backend_init();

            let model_params = llama_model_default_params();
            let model = llama_load_model_from_file(model_path_cstr.as_ptr(), model_params);
            if model.is_null() {
                llama_backend_free();
                return Err("Failed to load GGUF model".to_string());
            }

            let vocab = llama_model_get_vocab(model);
            if vocab.is_null() {
                llama_free_model(model);
                llama_backend_free();
                return Err("Failed to resolve model vocabulary".to_string());
            }

            let mut ctx_params = llama_context_default_params();
            ctx_params.n_ctx = n_ctx as u32;
            ctx_params.n_batch = n_ctx as u32;
            ctx_params.n_ubatch = n_ctx as u32;
            ctx_params.n_seq_max = MAX_SEQ_ID as u32;

            let threads = std::thread::available_parallelism()
                .map(|n| n.get() as i32)
                .unwrap_or(4)
                .max(1);
            ctx_params.n_threads = threads;
            ctx_params.n_threads_batch = threads;

            let ctx = llama_new_context_with_model(model, ctx_params);
            if ctx.is_null() {
                llama_free_model(model);
                llama_backend_free();
                return Err("Failed to create llama.cpp context".to_string());
            }

            llama_set_n_threads(ctx, threads, threads);

            Ok(Self {
                model,
                ctx,
                vocab,
                n_ctx,
            })
        }
    }

    pub fn infer(&mut self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        let prompt_cstr = CString::new(prompt).map_err(|_| {
            "Prompt contains interior null bytes which cannot be processed".to_string()
        })?;

        unsafe {
            let memory = llama_get_memory(self.ctx);
            if !memory.is_null() {
                llama_memory_clear(memory, true);
            }
        }

        let mut tokens: Vec<llama_token> = Vec::new();
        let mut capacity = prompt.len().max(1) + 8;

        loop {
            if capacity > i32::MAX as usize {
                return Err("Prompt too long for llama.cpp tokenizer".to_string());
            }

            tokens.resize(capacity, 0);

            let text_len = i32::try_from(prompt.len())
                .map_err(|_| "Prompt length exceeds supported limits".to_string())?;

            let n_tokens = unsafe {
                llama_tokenize(
                    self.vocab,
                    prompt_cstr.as_ptr(),
                    text_len,
                    tokens.as_mut_ptr(),
                    capacity as i32,
                    true,
                    false,
                )
            };

            if n_tokens >= 0 {
                tokens.truncate(n_tokens as usize);
                break;
            }

            capacity = capacity.saturating_mul(2);
        }

        if tokens.len() >= self.n_ctx.saturating_sub(32) as usize {
            let max_tokens = self.n_ctx.saturating_sub(32).max(1) as usize;
            if tokens.len() > max_tokens {
                let drop_count = tokens.len() - max_tokens;
                tokens.drain(0..drop_count);
            }
        }

        unsafe {
            self.decode_sequence(&tokens, 0)?;
        }

        let mut n_past = tokens.len() as i32;
        let mut generated = String::new();
        let eos_token = unsafe { llama_token_eos(self.vocab) };
        let vocab_size = unsafe { llama_n_vocab(self.vocab) } as usize;

        let mut decode_batch = unsafe { llama_batch_init(1, 0, MAX_SEQ_ID) };
        let mut decode_error: Option<String> = None;

        for _ in 0..max_tokens {
            let next_token = unsafe { self.sample_next_token(vocab_size, eos_token) };
            if next_token == eos_token {
                break;
            }

            let token_text = unsafe { self.token_to_string(next_token) };
            generated.push_str(&token_text);

            unsafe {
                decode_batch.n_tokens = 1;
                (*decode_batch.token) = next_token;
                (*decode_batch.pos) = n_past;
                (*decode_batch.n_seq_id) = 1;
                let seq_ptr = *decode_batch.seq_id;
                seq_ptr.write(0);
                (*decode_batch.logits) = 1;

                if llama_decode(self.ctx, decode_batch) != 0 {
                    if generated.trim().is_empty() {
                        decode_error =
                            Some("Model evaluation failed during generation".to_string());
                    }
                    break;
                }
            }

            n_past += 1;

            if generated.trim().is_empty() {
                continue;
            }

            if generated.ends_with('\n') && generated.lines().count() >= 2 {
                break;
            }
        }

        unsafe {
            llama_batch_free(decode_batch);
        }

        if let Some(err) = decode_error {
            return Err(err);
        }

        Ok(generated)
    }

    unsafe fn decode_sequence(&self, tokens: &[llama_token], start_pos: i32) -> Result<(), String> {
        if tokens.is_empty() {
            return Ok(());
        }

        let mut chunk_size = PROMPT_CHUNK_SIZE.max(1);
        let mut offset = 0usize;

        while offset < tokens.len() {
            let remaining = tokens.len() - offset;
            let current = remaining.min(chunk_size);
            let mut batch = llama_batch_init(current as i32, 0, MAX_SEQ_ID);

            for i in 0..current {
                let idx = i;
                (*batch.token.add(idx)) = tokens[offset + i];
                (*batch.pos.add(idx)) = start_pos + (offset + i) as i32;
                (*batch.n_seq_id.add(idx)) = 1;
                let seq_ptr = *batch.seq_id.add(idx);
                seq_ptr.write(0);
                let is_last = offset + i + 1 == tokens.len();
                (*batch.logits.add(idx)) = i8::from(is_last);
            }

            batch.n_tokens = current as i32;
            let status = llama_decode(self.ctx, batch);
            llama_batch_free(batch);

            if status != 0 {
                if chunk_size > 1 {
                    chunk_size = chunk_size.saturating_div(2).max(1);
                    continue;
                } else {
                    return Err("Model evaluation failed during prompt ingestion".to_string());
                }
            }

            offset += current;
        }

        Ok(())
    }

    unsafe fn sample_next_token(&self, vocab_size: usize, eos_token: llama_token) -> llama_token {
        let logits_ptr = llama_get_logits(self.ctx);
        if logits_ptr.is_null() {
            return eos_token;
        }

        let logits = std::slice::from_raw_parts(logits_ptr, vocab_size);
        let mut candidates: Vec<(llama_token, f32)> = logits
            .iter()
            .enumerate()
            .map(|(idx, &logit)| (idx as llama_token, logit))
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        let top_k = SAMPLING_TOP_K.max(1).min(candidates.len());
        candidates.truncate(top_k);

        let temperature = SAMPLING_TEMPERATURE.max(1e-5);
        let mut scaled = Vec::with_capacity(candidates.len());
        let mut max_logit = f32::NEG_INFINITY;
        for &(token, logit) in &candidates {
            let scaled_logit = logit / temperature;
            if scaled_logit > max_logit {
                max_logit = scaled_logit;
            }
            scaled.push((token, scaled_logit));
        }

        let mut weights = Vec::with_capacity(scaled.len());
        let mut weight_sum = 0.0f32;
        for (token, logit) in scaled {
            let weight = (logit - max_logit).exp();
            if weight.is_finite() && weight > 0.0 {
                weight_sum += weight;
                weights.push((token, weight));
            }
        }

        if weights.is_empty() {
            return candidates
                .first()
                .map(|(token, _)| *token)
                .unwrap_or(eos_token);
        }

        weights.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        let mut filtered = Vec::new();
        let mut cumulative = 0.0;
        for &(token, weight) in &weights {
            let prob = weight / weight_sum;
            cumulative += prob;
            filtered.push((token, weight));
            if SAMPLING_TOP_P < 1.0 && cumulative >= SAMPLING_TOP_P {
                break;
            }
        }

        if filtered.is_empty() {
            filtered.push(weights[0]);
        }

        if SAMPLING_MIN_P > 0.0 {
            let max_weight = filtered
                .iter()
                .map(|(_, weight)| *weight)
                .fold(f32::NEG_INFINITY, f32::max);
            let threshold = max_weight * SAMPLING_MIN_P;
            filtered.retain(|(_, weight)| *weight >= threshold);
            if filtered.is_empty() {
                filtered.push(weights[0]);
            }
        }

        let total_weight: f32 = filtered.iter().map(|(_, weight)| *weight).sum();
        if total_weight <= 0.0 {
            return filtered[0].0;
        }

        let mut rng = rand::rng();
        let mut sample = rng.random::<f32>() * total_weight;
        for (token, weight) in &filtered {
            sample -= *weight;
            if sample <= 0.0 {
                return *token;
            }
        }

        filtered
            .last()
            .map(|(token, _)| *token)
            .unwrap_or(eos_token)
    }

    unsafe fn token_to_string(&self, token: llama_token) -> String {
        let mut size = TOKEN_PIECE_INITIAL;
        let mut buffer: Vec<i8> = Vec::new();

        loop {
            if size > TOKEN_PIECE_MAX {
                return String::new();
            }

            buffer.resize(size, 0);
            let written =
                llama_token_to_piece(self.vocab, token, buffer.as_mut_ptr(), size as i32, 0, true);

            if written < 0 {
                size = size.saturating_mul(2);
                continue;
            }

            let written = written as usize;
            if written >= size {
                size = size.saturating_mul(2);
                continue;
            }

            let bytes: Vec<u8> = buffer[..written].iter().map(|b| *b as u8).collect();
            return String::from_utf8(bytes).unwrap_or_default();
        }
    }
}

impl Drop for LlamaSession {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx.is_null() {
                llama_free(self.ctx);
                self.ctx = ptr::null_mut();
            }

            if !self.model.is_null() {
                llama_free_model(self.model);
                self.model = ptr::null_mut();
            }

            llama_backend_free();
        }
    }
}
