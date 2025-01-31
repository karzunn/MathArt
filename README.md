# Overview

A Rust implementation of the algorithm used to generate the 'buddhabrot'. It utilizes parallel CPU computing and cycle detection to speed up rendering.

# Use

Use ```cargo run --release```, and the rendering will be saved as "output.png".

You can modify the following constants at the top of main.rs:
- MAP_RESOLUTION
    - This is the width and height in pixels of the output image.
- MAX_ITERATIONS
    - The maximum number of iterations to render for a given c value (location) on the complex plane (image).
- STEP
    - The distance used between c values along the complex plane when using the grid scan. The higher this number, the smoother the final image will be.
- SEGMENTS
    - The grid scan is parallelized along the y axis (imaginary axis). Each thread ends up storing what is essentially a partial image. If there are many threads due to a small STEP size, your computer's memory may be consumed by the partial images. We can split the y axis into segments instead, and combine all partial images before moving onto the next segment. This reduces memory usage, but will slow down the algorithm slightly.

If you fork this repo, you can use the "render" github action to generate renderings. This utilizes Github's runners to generate the renderings. Keep in mind however that Github's runners have a 6 hour maximum run time.

In order to store output images using the github action, you will need to do the following. 
- Configure the aws-actions/configure-aws-credentials@v2 action for your AWS account.
- Configure an S3 bucket and allow the role to put objects into it.
