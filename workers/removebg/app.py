import modal
from fastapi import Request

app = modal.App("nijika-removebg")

cache_volume = modal.Volume.from_name("nijika-model-cache", create_if_missing=True)

def download_model():
    import sys
    import types
    import os
    
    if os.path.exists("/cache/hub/models--ZhengPeng7--BiRefNet"):
        print("Model already exists in cache, skipping download.")
        return

    try:
        import torchvision.transforms.functional as F
        mod = types.ModuleType("torchvision.transforms.functional_tensor")
        for name in dir(F):
            setattr(mod, name, getattr(F, name))
        sys.modules["torchvision.transforms.functional_tensor"] = mod
    except ImportError:
        pass

    from transformers import AutoModelForImageSegmentation
    AutoModelForImageSegmentation.from_pretrained(
        "ZhengPeng7/BiRefNet", 
        trust_remote_code=True
    )

image = (
    modal.Image.debian_slim(python_version="3.11")
    .apt_install("libgl1-mesa-glx", "libglib2.0-0")
    .pip_install(
        "torch",
        "torchvision",
        "transformers",
        "accelerate",
        "timm"
    )
    .pip_install(
        "fastapi[standard]",
        "pillow",
        "einops",
        "requests",
        "kornia",
        "httpx"
    )
    .env({"HF_HOME": "/cache"})
    .run_function(download_model, volumes={"/cache": cache_volume})
)

@app.cls(
    image=image, 
    gpu="L4", 
    scaledown_window=300, 
    volumes={"/cache": cache_volume}
)
@modal.concurrent(max_inputs=8)
class Model:
    """
    BiRefNet model wrapper for background removal.
    Runs on Modal's serverless GPU infrastructure.
    """

    @modal.enter()
    def load_model(self):
        """
        Initializes the BiRefNet model and moves it to GPU.
        Executed once per container startup.
        """
        import sys
        import types
        import torch
        import time
        import os

        # Ensure model is downloaded (though it should be handled by Volume)
        if not os.path.exists("/cache/hub/models--ZhengPeng7--BiRefNet"):
            print("Model not found in Volume, downloading...")
            download_model()
            cache_volume.commit()

        try:
            import torchvision.transforms.functional as F
            mod = types.ModuleType("torchvision.transforms.functional_tensor")
            for name in dir(F):
                setattr(mod, name, getattr(F, name))
            sys.modules["torchvision.transforms.functional_tensor"] = mod
        except ImportError:
            pass

        from transformers import AutoModelForImageSegmentation
        from torchvision import transforms

        self.torch = torch
        self.transforms = transforms
        
        print("Loading BiRefNet model...")
        start = time.time()
        self.birefnet = AutoModelForImageSegmentation.from_pretrained(
            "ZhengPeng7/BiRefNet", 
            trust_remote_code=True,
            local_files_only=True
        )
        self.birefnet.to("cuda")
        self.birefnet.eval()
        print(f"Model loaded in {time.time() - start:.2f}s")
        
        self.transform_image = transforms.Compose([
            transforms.Resize((1024, 1024)),
            transforms.ToTensor(),
            transforms.Normalize([0.485, 0.456, 0.406], [0.229, 0.224, 0.225]),
        ])

    @modal.fastapi_endpoint(method="POST")
    async def remove(self, request: Request):
        """
        FastAPI endpoint to remove the background from an image.
        Accepts image URL in JSON or binary image data in request body.
        Returns a PNG image with a transparent background.
        """
        from fastapi import Response, HTTPException
        from PIL import Image
        import httpx
        import io

        content_type = request.headers.get("content-type", "")
        
        try:
            if "application/json" in content_type:
                body = await request.json()
                if "url" not in body:
                    raise HTTPException(status_code=400, detail="JSON body must contain 'url' field")
                
                image_url = body["url"]
                print(f"Fetching image from URL: {image_url}")
                async with httpx.AsyncClient() as client:
                    resp = await client.get(image_url, follow_redirects=True)
                    resp.raise_for_status()
                    image_bytes = resp.content
            else:
                print("Reading raw image bytes from request body")
                image_bytes = await request.body()

            if not image_bytes:
                raise HTTPException(status_code=400, detail="Empty image data")

            image = Image.open(io.BytesIO(image_bytes)).convert("RGB")
            original_size = image.size
            
            input_images = self.transform_image(image).unsqueeze(0).to("cuda")
            input_images = input_images.to(next(self.birefnet.parameters()).dtype)

            with self.torch.no_grad():
                preds = self.birefnet(input_images)[-1].sigmoid().cpu()
            
            pred = preds[0].squeeze()
            
            pred_pil = self.transforms.ToPILImage()(pred)
            mask = pred_pil.resize(original_size)
            
            image.putalpha(mask)
            
            output_buffer = io.BytesIO()
            image.save(output_buffer, format="PNG")
            output_buffer.seek(0)
            
            return Response(content=output_buffer.getvalue(), media_type="image/png")

        except HTTPException:
            raise
        except Exception as e:
            print(f"Error processing request: {str(e)}")
            raise HTTPException(status_code=500, detail=str(e))
