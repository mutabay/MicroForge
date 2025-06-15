# MicroForge ⚙️

**LLM-Powered Automation for Modern Microservice Architectures**

---

## 🧭 Overview

**MicroForge** is a developer-focused platform that automatically generates, validates, and deploys microservices from high-level descriptions using **Large Language Models (LLMs)** and **OpenAPI contracts**. It combines the runtime efficiency of **Rust** with the flexibility of **Python agents** to enable rapid, scalable development of robust service-based systems.

The project is built to explore and develop skills across:
- Rust backend development (Axum, Tokio)
- Distributed systems and microservice design
- Software architecture and automation
- Web development with OpenAPI-first philosophy
- Multi-agent systems and AI orchestration using Python

---

## 🎯 Project Goals

- **Automate microservice development** from user intent (prompt) to production-ready service.
- **Use OpenAPI as the source of truth**, enabling clear contracts and safe integrations.
- **Generate and evolve services using LLM-powered agents** that handle API spec creation, code scaffolding, validation, and CI suggestions.
- **Design a central Rust-based system** that efficiently runs and connects services using both synchronous (HTTP/gRPC) and asynchronous (event-based) communication models.
- **Improve developer productivity** by reducing manual work in CRUD setup, routing, and boilerplate orchestration logic.

---


## 🔧 What This Application Will Do

1. **Accept high-level user input** describing a desired microservice (e.g., “Create a service to manage products with SKU, name, and stock”).
2. **Use an LLM agent to generate an OpenAPI contract** defining the API.
3. **Automatically scaffold a Rust microservice** using frameworks like Axum and annotate it with `utoipa` or similar tools to match the OpenAPI spec.
4. **Run automated tests and contract validation**, using tools like `cargo test` and Schemathesis.
5. **Optionally containerize and register the service** for communication within a distributed system.
6. **Support real-time (sync) and event-based (async) service interactions**, improving runtime responsiveness and system scalability.

---

## 📋 Requirements (High-Level)

| Component | Technology |
|-----------|------------|
| Runtime Backend | **Rust**, using Axum + Tokio |
| API Contracts | **OpenAPI 3.0** |
| LLM Orchestration | **Python**, using OpenAI or local LLM APIs |
| Agent Framework | Modular Python scripts or threaded orchestration |
| Testing | `cargo test`, `schemathesis`, or contract-driven test tools |
| CI/CD (Optional) | GitHub Actions or Docker pipelines |

---
## 🛣️ Next Steps

1. **Initialize a Rust workspace** with basic Axum setup.
2. **Set up Python CLI** to handle user prompts and agent logic.
3. **Integrate OpenAPI generation & Rust code templating** based on specs.
4. **Define communication model** (sync/async) and inter-service handling.
5. **Develop one full cycle**: prompt → spec → service → test → deploy.
6. **Iterate to build service registry and LLM feedback/refinement loop**.

---

## ⚙️ Core Features

| Feature | Description |
|--------|-------------|
| 🧠 Prompt-to-Service | Generate a production-ready microservice from a high-level prompt |
| 📜 OpenAPI Spec Generation | API contract is the first artifact; everything is scaffolded from it |
| ⚙️ Rust-based Runtime | Highly efficient services built with Axum, Tokio, and utoipa |
| 🤖 Multi-Agent AI Orchestration | Python agents manage spec generation, code creation, testing, and deployment |
| 🔁 Sync/Async Communication | Both HTTP and message-based (Kafka/RabbitMQ) paths supported |
| ✅ Contract Validation | Automatic validation against OpenAPI using Schemathesis |
| 📦 Docker + CI Support | Each microservice is packaged and CI-tested for rapid deployment |

---

## 📋 Technical Stack

| Layer | Technology |
|-------|------------|
| Language (runtime) | **Rust** |
| API framework | **Axum**, **utoipa** |
| Async runtime | **Tokio** |
| AI agents | **Python**, OpenAI API, async orchestration |
| OpenAPI tooling | openapi-generator, schemathesis, utoipa |
| Containerization | Docker |
| Messaging (Async) | Kafka or RabbitMQ |
| CI/CD | GitHub Actions, Docker Compose |

---

## 🧠 LLM Agent Design

### 🧾 Spec Agent
- Input: User prompt
- Output: OpenAPI 3.0 spec
- Tools: LLM + OpenAPI template knowledge

### 🛠️ Code Agent
- Input: OpenAPI spec
- Output: Rust Axum project (annotated)
- Tasks:
  - Generate routes, handlers
  - Inject `utoipa` annotations
  - Setup basic CRUD logic

### 🧪 Validation Agent
- Input: Rust service
- Output: Test report
- Tasks:
  - Run `cargo check`, `cargo test`
  - Run `schemathesis` against OpenAPI
  - Return errors to refine code if needed

---

## 🚧 Development Roadmap

### Phase 0 – Environment Setup
- [x] Set up GitHub repo
- [x] Define tech stack
- [x] Set up local dev environment for Rust and Python

### Phase 1 – MVP Pipeline
- [ ] Implement `spec_agent.py`
- [ ] Implement `code_agent.py` (minimal handler + route)
- [ ] Generate first Axum service from prompt + OpenAPI
- [ ] Run manual tests (cargo + postman)
- [ ] Use `utoipa` to auto-generate OpenAPI doc

### Phase 2 – Automation + Testing
- [ ] Add `validation_agent.py`
- [ ] Integrate Schemathesis + cargo test
- [ ] Feedback loop to improve LLM prompts

### Phase 3 – Runtime & Communication
- [ ] Add async event-based communication (Kafka)
- [ ] Add Rust runtime “core” app (service discovery + router)
- [ ] Support cross-service communication

### Phase 4 – Registry & Deployment
- [ ] Create service registry
- [ ] Add Dockerfile to each generated service
- [ ] Add GitHub Actions workflows

### Phase 5 – Final Polish
- [ ] Add documentation & CLI flags
- [ ] Handle agent prompt errors & retries
- [ ] Optionally add front-end for visualization

---

## 🎓 Learning Outcomes

| Skill Domain | What You'll Learn |
|--------------|-------------------|
| Rust | Building async APIs, Axum, memory-safe concurrency |
| Python | Multi-agent orchestration, OpenAI API, automation scripting |
| AI | Prompt engineering, multi-step workflows, agent design |
| Web APIs | OpenAPI spec design, RESTful patterns |
| Software Architecture | Modular design, service registries, deployment pipelines |
| Distributed Systems | Sync/async messaging, Kafka integration, service isolation |
| DevOps | CI/CD pipelines, Dockerization, GitHub Actions |

---

## 📌 Key Constraints & Assumptions

- The user input must be structured enough for reliable LLM spec generation.
- All services follow OpenAPI contract-first principles.
- Rust is the execution runtime; Python is only used for orchestration logic.
- Stateless service design is preferred for parallel execution and scale-out.

---

## 📄 References

- [arXiv:2502.09766v1](https://arxiv.org/html/2502.09766v1) – Chauhan et al., LLM-Generated Microservice Implementations  

---
