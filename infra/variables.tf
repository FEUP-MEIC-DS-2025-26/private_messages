variable "project_id" {
  description = "The Google Cloud project ID."
  type        = string
}

variable "region" {
  description = "The region where resources will be deployed."
  type        = string
  default     = "europe-southwest1"
}

variable "bucket_name" {
  description = "The name for the Google Storage Bucket where the database is deployed."
  type        = string
}

variable "frontend_name" {
  description = "The name for the frontend Cloud Run service."
  type        = string
}

variable "frontend_image" {
  description = "Frontend container image to deploy to Cloud Run."
  type        = string
}

variable "backend_name" {
  description = "The name for the backend Cloud Run service."
  type        = string
}

variable "backend_image" {
  description = "Backend container image to deploy to Cloud Run."
  type        = string
}
