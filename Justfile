test:
    ./scripts/tests.sh

citest:
  cargo nextest run


# https://www.reddit.com/r/docker/comments/uencaa/create_docker_image_on_m1_mac_for_x86/
push:
  docker buildx build --platform linux/amd64 \
    -f docker/Dockerfile.localprod \
    -t us.gcr.io/trainton-ddd5c/nautilus:1.0 \
    —-push . \
