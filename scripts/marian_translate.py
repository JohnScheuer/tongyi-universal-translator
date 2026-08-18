import sys
import json
import os
import traceback

# Requer:
#   pip install torch transformers sentencepiece
from transformers import MarianMTModel, MarianTokenizer
import torch

torch.set_num_threads(1)

# Cache por modelo (src -> zh)
_MODELS = {}

def _model_dir_for_pair(model_root: str, source: str, target: str) -> str:
    # target esperado: zh-cn
    # diretórios que vamos usar:
    #   <model_root>/opus-mt-pt-zh
    #   <model_root>/opus-mt-en-zh
    #   <model_root>/opus-mt-es-zh
    target = target.lower()
    if target not in ["zh-cn", "zh"]:
        raise ValueError(f"Unsupported target: {target}")

    source = source.lower()
    if source == "pt":
        sub = "opus-mt-pt-zh"
    elif source == "en":
        sub = "opus-mt-en-zh"
    elif source == "es":
        sub = "opus-mt-es-zh"
    else:
        raise ValueError(f"Unsupported source: {source}")

    return os.path.join(model_root, sub)

def _load_model(model_root: str, source: str, target: str):
    key = (model_root, source.lower(), target.lower())
    if key in _MODELS:
        return _MODELS[key]

    local_dir = _model_dir_for_pair(model_root, source, target)
    if not os.path.isdir(local_dir):
        raise FileNotFoundError(
            f"Model directory not found: {local_dir}. "
            f"Run scripts/marian_download_models.py --model-dir \"{model_root}\" first."
        )

    tokenizer = MarianTokenizer.from_pretrained(local_dir)
    model = MarianMTModel.from_pretrained(local_dir)
    model.eval()

    _MODELS[key] = (tokenizer, model)
    return tokenizer, model

def _translate(model_root: str, text: str, source: str, target: str) -> str:
    tokenizer, model = _load_model(model_root, source, target)

    # MarianMT funciona melhor com textos curtos/médios.
    # V1: não fazer segmentação complexa aqui.
    batch = tokenizer([text], return_tensors="pt", padding=True, truncation=True)

    with torch.no_grad():
        gen = model.generate(
            **batch,
            num_beams=4,
            max_new_tokens=256,
        )

    out = tokenizer.batch_decode(gen, skip_special_tokens=True)[0]
    return out

def _write(obj):
    sys.stdout.write(json.dumps(obj, ensure_ascii=False) + "\n")
    sys.stdout.flush()

def main():
    # Protocolo: JSON por linha em stdin
    # Request:
    #   {"text":"...", "source":"pt|en|es", "target":"zh-cn", "model_root":"C:\\...\\models"}
    # Response:
    #   {"ok":true, "translated":"..."}
    # ou {"ok":false, "error":"..."}

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            req = json.loads(line)

            if req.get("cmd") == "ping":
                _write({"ok": True, "pong": True})
                continue

            text = req.get("text", "")
            source = req.get("source", "pt")
            target = req.get("target", "zh-cn")
            model_root = req.get("model_root", "./models")

            if not isinstance(text, str) or not text.strip():
                _write({"ok": False, "error": "empty text"})
                continue

            translated = _translate(model_root, text, source, target)
            _write({"ok": True, "translated": translated})

        except Exception as e:
            # Envia erro legível (sem stack gigante por padrão)
            _write({"ok": False, "error": str(e)})

if __name__ == "__main__":
    main()
