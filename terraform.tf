terraform {
  required_providers {
    google = {
      source = "hashicorp/google"
      version = "7.8.0"
    }
  }
}

provider "google" {
  project = "ds-2025-g51-private-messages"
  region = "europe-southwest1"
  zone = "europe-southwest1-b"
}

resource "google_cloud_run_v2_service" "backend" {
  name     = "backend-private-messages"
  location = "europe-southwest1"
  deletion_protection = false

  template {
    containers {
      image = "europe-southwest1-docker.pkg.dev/ds-2025-g51-private-messages/backend/production"
      ports {
        container_port = 8080
      }
    }
  }
}

resource "google_cloud_run_v2_service_iam_binding" "backend_public_access" {
  project  = google_cloud_run_v2_service.backend.project
  location = google_cloud_run_v2_service.backend.location
  name     = google_cloud_run_v2_service.backend.name

  role   = "roles/run.invoker"
  members = ["allUsers"]
}

output "backend_url" {
  description = "The URL of the deployed backend Cloud Run service."
  value       = google_cloud_run_v2_service.backend.uri
}


resource "google_cloud_run_v2_service" "frontend" {
  name     = "frontend-private-messages"
  location = "europe-southwest1"
  deletion_protection = false

  template {
    containers {
      image = "europe-southwest1-docker.pkg.dev/ds-2025-g51-private-messages/frontend/production"
      ports {
        container_port = 3001
      }
      env {
        name  = "BACKEND_URL"
        value = google_cloud_run_v2_service.backend.uri
      }
    }
  }
}

resource "google_cloud_run_v2_service_iam_binding" "frontend_public_access" {
  project  = google_cloud_run_v2_service.frontend.project
  location = google_cloud_run_v2_service.frontend.location
  name     = google_cloud_run_v2_service.frontend.name

  role   = "roles/run.invoker"
  members = ["allUsers"]
}

output "frontend_url" {
  description = "The URL of the deployed frontend Cloud Run service."
  value       = google_cloud_run_v2_service.frontend.uri
}