# Nautilus

### Backend powering [Trainton](https://trainton.com/beta)

You can find the [ WIP Blog ](https://www.jacobmaizel.com/projects/training-platform) here.

Written in Rust and Axum 🦀

Feel free to reference or use some of the code for your own projects.

Features:

-   Pagination [ref](./src/pagination.rs)
-   Opentelemetry tracing [ref](./src/telemetry.rs) with configurations for dev and prod using [ Cloud Trace ](https://cloud.google.com/trace)
-   Authentication Middleware [ref] (./src/auth.rs)
-   CI/CD Pipeline [ref](./.github/workflows/cicd.yml) authenticating and pushing to [Artifact Registry](https://cloud.google.com/artifact-registry).
-   Environment specific config files and loading [ref](./src/settings.rs)
-   Custom Axum Extractors [ref](./src/util/extractors.rs)
