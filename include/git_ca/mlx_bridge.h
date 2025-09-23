#pragma once

#include "rust/cxx.h"

namespace git_ca {

// Minimal placeholder bridge for MLX inference.
// The actual implementation will wire into MLX once the runtime
// dependencies are available on the build machine.
void load_model(::rust::Str model_path);
::rust::String generate_commit(::rust::Str diff_text);

}  // namespace git_ca
