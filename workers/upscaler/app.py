import modal
from fastapi import Request

app = modal.App("nijika-upscaler")

cache_volume = modal.Volume.from_name("nijika-model-cache", create_if_missing=True)

def download_models():
    import sys
    import types
    import os
    try:
        import torchvision.transforms.functional as F
        mod = types.ModuleType("torchvision.transforms.functional_tensor")
        for name in dir(F):
            setattr(mod, name, getattr(F, name))
        sys.modules["torchvision.transforms.functional_tensor"] = mod
    except ImportError:
        pass

    from basicsr.utils.download_util import load_file_from_url
    
    os.makedirs("/cache/weights", exist_ok=True)
    
    models = [
        ('https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.2.4/RealESRGAN_x4plus_anime_6B.pth', 'RealESRGAN_x4plus_anime_6B.pth'),
        ('https://github.com/TencentARC/GFPGAN/releases/download/v1.3.0/GFPGANv1.3.pth', 'GFPGANv1.3.pth'),
        ('https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.0/RealESRGAN_x4plus.pth', 'RealESRGAN_x4plus.pth'),
        ('https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.1/RealESRNet_x4plus.pth', 'RealESRNet_x4plus.pth'),
        ('https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.1/RealESRGAN_x2plus.pth', 'RealESRGAN_x2plus.pth'),
        ('https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-general-x4v3.pth', 'realesr-general-x4v3.pth'),
        ('https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-general-wdn-x4v3.pth', 'realesr-general-wdn-x4v3.pth'),
    ]
    
    for url, file_name in models:
        path = os.path.join('/cache/weights', file_name)
        if not os.path.exists(path):
            print(f"Downloading {file_name}...")
            load_file_from_url(
                url=url,
                model_dir='/cache/weights',
                progress=True,
                file_name=file_name
            )

image = (
    modal.Image.debian_slim(python_version="3.11")
    .apt_install("libgl1-mesa-glx", "libglib2.0-0")
    .pip_install(
        "torch",
        "torchvision",
        "numpy",
        "opencv-python-headless",
        "basicsr",
        "realesrgan",
        "gfpgan",
        "fastapi[standard]",
        "pillow",
        "requests",
        "httpx",
        "setuptools==69.5.1"
    )
    .env({"HF_HOME": "/cache"})
    .run_function(download_models, volumes={"/cache": cache_volume})
)

@app.cls(
    image=image, 
    gpu="L4", 
    scaledown_window=300, 
    volumes={"/cache": cache_volume}
)
@modal.concurrent(max_inputs=4)
class Upscaler:
    @modal.enter()
    def load_model(self):
        import sys
        import types
        import os
        import threading
        
        # Ensure models are downloaded to Volume
        if not os.path.exists("/cache/weights") or len(os.listdir("/cache/weights")) < 7:
            print("Models missing from Volume, downloading...")
            download_models()
            cache_volume.commit()

        try:
            import torchvision.transforms.functional as F
            mod = types.ModuleType("torchvision.transforms.functional_tensor")
            for name in dir(F):
                setattr(mod, name, getattr(F, name))
            sys.modules["torchvision.transforms.functional_tensor"] = mod
        except ImportError:
            pass

        self.UPSAMPLER_CACHE = {}
        self.GFPGAN_FACE_ENHANCER = {}
        self.cache_lock = threading.Lock()
        self.DEVICE = "cuda"
        self.USE_HALF = True
        
        if not os.path.exists("/cache/weights"):
            os.makedirs("/cache/weights", exist_ok=True)

    def get_model_and_paths(self, model_name, denoise_strength):
        import os
        from basicsr.archs.rrdbnet_arch import RRDBNet
        from realesrgan.archs.srvgg_arch import SRVGGNetCompact
        from basicsr.utils.download_util import load_file_from_url

        if model_name in ('RealESRGAN_x4plus', 'RealESRNet_x4plus'):
            model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=4)
            netscale = 4
            file_url = ['https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.0/RealESRGAN_x4plus.pth'] if model_name == 'RealESRGAN_x4plus' else ['https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.1/RealESRNet_x4plus.pth']
        elif model_name == 'RealESRGAN_x4plus_anime_6B':
            model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=6, num_grow_ch=32, scale=4)
            netscale = 4
            file_url = ['https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.2.4/RealESRGAN_x4plus_anime_6B.pth']
        elif model_name == 'RealESRGAN_x2plus':
            model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=2)
            netscale = 2
            file_url = ['https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.1/RealESRGAN_x2plus.pth']
        elif model_name == 'realesr-general-x4v3':
            model = SRVGGNetCompact(num_in_ch=3, num_out_ch=3, num_feat=64, num_conv=32, upscale=4, act_type='prelu')
            netscale = 4
            file_url = [
                'https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-general-wdn-x4v3.pth',
                'https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-general-x4v3.pth'
            ]
        else:
            raise ValueError(f"Unsupported model: {model_name}")

        model_path = os.path.join('/cache/weights', model_name + '.pth')
        if not os.path.isfile(model_path):
            for url in file_url:
                model_path = load_file_from_url(url=url, model_dir='/cache/weights', progress=True, file_name=None)

        dni_weight = None
        if model_name == 'realesr-general-x4v3' and denoise_strength != 1:
            wdn_model_path = model_path.replace('realesr-general-x4v3', 'realesr-general-wdn-x4v3')
            model_path = [model_path, wdn_model_path]
            dni_weight = [denoise_strength, 1 - denoise_strength]

        return model, netscale, model_path, dni_weight

    def get_upsampler(self, model_name, denoise_strength):
        key = (model_name, float(denoise_strength))
        with self.cache_lock:
            if key in self.UPSAMPLER_CACHE:
                return self.UPSAMPLER_CACHE[key]

        from realesrgan import RealESRGANer
        model, netscale, model_path, dni_weight = self.get_model_and_paths(model_name, denoise_strength)

        upsampler = RealESRGANer(
            scale=netscale,
            model_path=model_path,
            dni_weight=dni_weight,
            model=model,
            tile=0,
            tile_pad=10,
            pre_pad=10,
            half=self.USE_HALF,
            gpu_id=0
        )
        with self.cache_lock:
            self.UPSAMPLER_CACHE[key] = upsampler
        return upsampler

    def get_face_enhancer(self, upsampler, outscale):
        import os
        key = int(outscale)
        with self.cache_lock:
            if key in self.GFPGAN_FACE_ENHANCER:
                return self.GFPGAN_FACE_ENHANCER[key]
        from gfpgan import GFPGANer
        
        gfpgan_path = '/cache/weights/GFPGANv1.3.pth'
        face_enhancer = GFPGANer(
            model_path=gfpgan_path if os.path.exists(gfpgan_path) else 'https://github.com/TencentARC/GFPGAN/releases/download/v1.3.0/GFPGANv1.3.pth',
            upscale=int(outscale),
            arch='clean',
            channel_multiplier=2,
            bg_upsampler=upsampler
        )
        with self.cache_lock:
            self.GFPGAN_FACE_ENHANCER[key] = face_enhancer
        return face_enhancer

    @modal.fastapi_endpoint(method="POST")
    async def upscale(self, request: Request):
        import io
        import cv2
        import numpy as np
        import httpx
        from fastapi import Response, HTTPException
        from PIL import Image

        content_type = request.headers.get("content-type", "")
        
        # Default parameters
        model_name = "RealESRGAN_x4plus_anime_6B"
        denoise_strength = 0.5
        face_enhance = False
        outscale = 4

        try:
            if "application/json" in content_type:
                body = await request.json()
                if "url" not in body:
                    raise HTTPException(status_code=400, detail="JSON body must contain 'url' field")
                
                image_url = body["url"]
                model_name = body.get("model", model_name)
                face_enhance = body.get("face_enhance", face_enhance)
                outscale = body.get("scale", outscale)
                
                print(f"Fetching image from URL: {image_url}")
                async with httpx.AsyncClient() as client:
                    resp = await client.get(image_url, follow_redirects=True)
                    resp.raise_for_status()
                    image_bytes = resp.content
            else:
                image_bytes = await request.body()
                model_name = request.headers.get("X-Model", model_name)
                face_enhance = request.headers.get("X-Face-Enhance", "false").lower() == "true"
                outscale = int(request.headers.get("X-Scale", str(outscale)))

            if not image_bytes:
                raise HTTPException(status_code=400, detail="Empty image data")

            img = Image.open(io.BytesIO(image_bytes))
            upsampler = self.get_upsampler(model_name, denoise_strength)

            cv_img = np.array(img)
            if cv_img.ndim == 3 and cv_img.shape[2] == 4:
                img_bgra = cv2.cvtColor(cv_img, cv2.COLOR_RGBA2BGRA)
            elif cv_img.ndim == 3 and cv_img.shape[2] == 3:
                bgr = cv2.cvtColor(cv_img, cv2.COLOR_RGB2BGR)
                alpha = np.full((bgr.shape[0], bgr.shape[1], 1), 255, dtype=bgr.dtype)
                img_bgra = np.concatenate([bgr, alpha], axis=2)
            else:
                bgr = cv2.cvtColor(cv_img, cv2.COLOR_GRAY2BGR)
                alpha = np.full((bgr.shape[0], bgr.shape[1], 1), 255, dtype=bgr.dtype)
                img_bgra = np.concatenate([bgr, alpha], axis=2)

            if face_enhance:
                face_enhancer = self.get_face_enhancer(upsampler, outscale)
                _, _, output = face_enhancer.enhance(
                    img_bgra, has_aligned=False, only_center_face=False, paste_back=True
                )
            else:
                output, _ = upsampler.enhance(img_bgra, outscale=int(outscale))

            if output.ndim == 3 and output.shape[2] == 4:
                output_to_save = cv2.cvtColor(output, cv2.COLOR_BGRA2BGR)
            elif output.ndim == 3 and output.shape[2] == 3:
                output_to_save = output
            else:
                output_to_save = cv2.cvtColor(output, cv2.COLOR_GRAY2BGR)
            
            _, buffer = cv2.imencode(".jpg", output_to_save, [cv2.IMWRITE_JPEG_QUALITY, 95])
            
            return Response(content=buffer.tobytes(), media_type="image/jpeg")

        except HTTPException:
            raise
        except Exception as e:
            print(f"Error processing request: {str(e)}")
            import traceback
            traceback.print_exc()
            raise HTTPException(status_code=500, detail=str(e))
