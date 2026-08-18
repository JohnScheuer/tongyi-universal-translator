import argparse
import os

from transformers import MarianMTModel, MarianTokenizer

MODELS = {
    "pt": "Helsinki-NLP/opus-mt-pt-zh",
    "en": "Helsinki-NLP/opus-mt-en-zh",
    "es": "Helsinki-NLP/opus-mt-es-zh",
}

DIRS = {
    "pt": "opus-mt-pt-zh",
    "en": "opus-mt-en-zh",
    "es": "opus-mt-es-zh",
}

def download_one(model_root: str, src: str):
    name = MODELS[src]
    out_dir = os.path.join(model_root, DIRS[src])
    os.makedirs(out_dir, exist_ok=True)

    print(f"Downloading {name} -> {out_dir}")
    tok = MarianTokenizer.from_pretrained(name)
    mdl = MarianMTModel.from_pretrained(name)

    tok.save_pretrained(out_dir)
    mdl.save_pretrained(out_dir)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--model-dir", default="./models", help="Onde salvar os modelos (default: ./models)")
    ap.add_argument("--langs", default="pt,en,es", help="Quais sources baixar: pt,en,es (default: pt,en,es)")
    args = ap.parse_args()

    model_root = args.model_dir
    langs = [x.strip() for x in args.langs.split(",") if x.strip()]

    for lang in langs:
        if lang not in MODELS:
            raise SystemExit(f"Unsupported lang: {lang}")
        download_one(model_root, lang)

    print("Done.")

if __name__ == "__main__":
    main()
