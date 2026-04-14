FROM python:3.12-slim

WORKDIR /app

COPY requirements.txt .
ENV PIP_DISABLE_PIP_VERSION_CHECK=1
ENV PIP_DEFAULT_TIMEOUT=120
RUN python -m pip install --no-cache-dir --retries 10 -r requirements.txt

COPY app/ ./app/

ENV PORT=8000
ENV ENGINE_API_BASE_URL=http://engine-api:8080

EXPOSE 8000
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
