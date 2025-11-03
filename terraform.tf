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

resource "google_cloud_run_v2_service" "default" {
  name     = "ds-2025-g51-private-messages"
  location = "europe-southwest1"
  deletion_protection = false

  template {
    containers {
      image = "europe-southwest1-docker.pkg.dev/ds-2025-g51-private-messages/prototype/production"
      ports {
        container_port = 8080
      }
    }
  }
}

resource "google_cloud_run_v2_service_iam_binding" "public_access" {
  project  = google_cloud_run_v2_service.default.project
  location = google_cloud_run_v2_service.default.location
  name     = google_cloud_run_v2_service.default.name

  role   = "roles/run.invoker"
  members = ["allUsers"]
}

output "service_url" {
  description = "The URL of the deployed Cloud Run service."
  value       = google_cloud_run_v2_service.default.uri
}
