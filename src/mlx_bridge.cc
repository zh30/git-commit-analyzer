#include "git_ca/mlx_bridge.h"

#include <string>

namespace git_ca {
namespace {
bool g_model_loaded = false;
}

void load_model(::rust::Str model_path) {
    // Placeholder: real implementation will initialize MLX and load
    // a quantized model artifact located at model_path.
    // Avoid unused parameter warnings until MLX is hooked up.
    (void)model_path;
    g_model_loaded = true;
}

::rust::String generate_commit(::rust::Str diff_text) {
    // TODO: Replace with MLX-backed inference. For now we surface a
    // deterministic placeholder so the Rust side can validate the flow
    // without shipping Python.
    if (!g_model_loaded) {
        return ::rust::String("[mlx-error: model not initialized]");
    }
    std::string prefix = "[mlx-placeholder] ";
    std::string diff_str(diff_text.begin(), diff_text.end());
    if (diff_str.size() > 48) {
        diff_str.resize(48);
    }
    return ::rust::String(prefix + diff_str);
}

}  // namespace git_ca
