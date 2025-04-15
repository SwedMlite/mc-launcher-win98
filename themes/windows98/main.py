import os
from PIL import Image # Need to install Pillow: pip install Pillow

# Get current directory
current_dir = os.getcwd()

# List files in the directory
files = os.listdir(current_dir)

# Define common image extensions (lowercase)
image_extensions = ['.jpg', '.jpeg', '.png', '.gif', '.bmp', '.tiff', '.webp']

print(f"Searching for images in: {current_dir}")

# Loop through files
for filename in files:
    # Construct full path
    full_path = os.path.join(current_dir, filename)

    # Check if it's a file and has an image extension
    if os.path.isfile(full_path):
        # Get extension and convert to lowercase
        _, ext = os.path.splitext(filename)
        ext_lower = ext.lower()

        if ext_lower in image_extensions:
            # Try to open the image and get dimensions
            try:
                with Image.open(full_path) as img: # Use 'with' for automatic closing
                    width, height = img.size
                    print(f"Файл: {filename}, Разрешение: {width}x{height}")
            except Exception as e:
                # Handle errors (e.g., corrupted file, non-image file with image extension)
                print(f"Не удалось обработать файл: {filename} - Ошибка: {e}")

print("\nПоиск завершен.")