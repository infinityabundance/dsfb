# Architecture

## Logic flow

```mermaid
flowchart LR
    A[Observation / Prediction] --> B[Residual]
    B --> C[Sign]
    C --> D[Syntax]
    D --> E[Grammar]
    E --> F[Semantics]
    F --> G[Constrained Output]
