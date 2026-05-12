# MicroForge Core Demo

This demo shows the two main capabilities.

## 1. Service Creation

Request:

```text
POST /forge
```

Demo prompt:

```text
A car rental service with cars, bookings, customers, pickup date, return date, and booking status
```

What it shows:

MicroForge takes a plain-English service description and asks the configured LLM to generate an OpenAPI 3.0 specification.

In the response, point to:

```text
provider
openapi_spec
paths
components/schemas
```

Clean view:

After running this request in Postman, click the **Visualize** tab. The collection automatically formats the generated OpenAPI YAML there.

You do not need to manually copy, replace `\n`, or remove markdown fences.

Important: this request requires an LLM provider. For local Ollama, run:

```powershell
ollama serve
ollama pull llama3.1:8b
```

With Docker Compose, `.env` should use:

```text
OLLAMA_URL=http://host.docker.internal:11434
```

## 2. Service Coordination

Request:

```text
POST /pipelines
```

Demo workflow:

```text
check API health
check car availability
  -> calculate price
  -> create booking summary
  -> confirm booking
```

What it shows:

MicroForge executes a multi-step workflow as a DAG. Independent tasks start in parallel. Dependent tasks wait for the tasks they need.

## Run The App

```powershell
docker compose up --build
```

## Use Postman

Import:

```text
demo/MicroForge.postman_collection.json
```

Run:

1. `Health Check`
2. `Service Creation - Generate OpenAPI Spec`
3. `Service Coordination - Run Booking Workflow`
4. `Pipeline History`

If the LLM is not running, step 2 will fail. That is expected. Step 3 still demonstrates the orchestration engine.
