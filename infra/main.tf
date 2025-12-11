resource "google_cloud_run_v2_service" "backend" {
  name     = var.backend_name
  location = var.region

  template {
    containers {
      image = var.backend_image
      
      ports {
        container_port = 8080
      }
      
      env {
        name = "GOOGLE_APPLICATION_CREDENTIALS"
        value = "./local/pubsub.json"
      }
    }
  }
}

// TODO: Restrict backend so that it is only accessible by the gateway
resource "google_cloud_run_v2_service_iam_binding" "backend_public_access" {
  project  = google_cloud_run_v2_service.backend.project
  location = google_cloud_run_v2_service.backend.location
  name     = google_cloud_run_v2_service.backend.name
  role    = "roles/run.invoker"
  members = ["allUsers"]
}

output "backend_url" {
  description = "The URL of the deployed backend Cloud Run service."
  value       = google_cloud_run_v2_service.backend.uri
}

resource "google_cloud_run_v2_service" "frontend" {
  name     = var.frontend_name
  location = var.region

  template {
    containers {
      image = var.frontend_image

      ports {
        container_port = 3001
      }

      env {
        name  = "PUBLIC_BACKEND_URL"
        value = "https://api.madeinportugal.store"
      }
    }
  }
}

resource "google_cloud_run_v2_service_iam_binding" "frontend_public_access" {
  project  = google_cloud_run_v2_service.frontend.project
  location = google_cloud_run_v2_service.frontend.location
  name     = google_cloud_run_v2_service.frontend.name
  role    = "roles/run.invoker"
  members = ["allUsers"]
}

output "frontend_url" {
  description = "The URL of the deployed frontend Cloud Run service."
  value       = google_cloud_run_v2_service.frontend.uri
}
