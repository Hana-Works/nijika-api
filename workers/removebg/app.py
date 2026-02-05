import modal
import io
import time
from fastapi import Request, Response, HTTPException

app = modal.App("nijika-removebg")
volume = modal.Volume.from_name("nijika-removebg-volume", create_if_missing=True)

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
        "kornia"
    )
    .env({"HF_HOME": "/cache"})
)

@app.cls(image=image, gpu="T4", scaledown_window=300, volumes={"/cache": volume})
class Model:
    @modal.enter()
    def load_model(self):
        import torch
        from transformers import AutoModelForImageSegmentation
        from torchvision import transforms

        self.torch = torch
        self.transforms = transforms
        
        print("Loading BiRefNet model...")
        start = time.time()
        self.birefnet = AutoModelForImageSegmentation.from_pretrained(
            "ZhengPeng7/BiRefNet", 
            trust_remote_code=True
        )
        self.birefnet.to("cuda")
        self.birefnet.eval()
        print(f"Model loaded in {time.time() - start:.2f}s")
        
        if hasattr(volume, "commit"):
            volume.commit()

        self.transform_image = transforms.Compose([
            transforms.Resize((1024, 1024)),
            transforms.ToTensor(),
            transforms.Normalize([0.485, 0.456, 0.406], [0.229, 0.224, 0.225]),
        ])

    @modal.fastapi_endpoint(method="POST")
    async def remove(self, request: Request):
        from PIL import Image
        import requests

        content_type = request.headers.get("content-type", "")
        
        try:
            if "application/json" in content_type:
                body = await request.json()
                if "url" not in body:
                     raise HTTPException(status_code=400, detail="JSON body must contain 'url' field")
                
                image_url = body["url"]
                print(f"Fetching image from URL: {image_url}")
                resp = requests.get(image_url, stream=True)
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
