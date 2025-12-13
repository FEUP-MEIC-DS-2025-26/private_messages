RAND_SUFFIX="$(openssl rand -hex 8)"
FRONTEND_DOCKER_URL="europe-southwest1-docker.pkg.dev/ds-2025-g51-private-messages/frontend/production:$RAND_SUFFIX"
BACKEND_DOCKER_URL="europe-southwest1-docker.pkg.dev/ds-2025-g51-private-messages/backend/production:$RAND_SUFFIX"
export JUMPSELLER=$(cat ../local/jumpseller_cred.json)
export PUBSUB=$(cat ../local/pubsub.json)
export PASSWORD=$(cat ../local/password.txt)
export SALT=$(cat ../local/salt.txt)

cd ../frontend
docker build --no-cache -t $FRONTEND_DOCKER_URL .
docker push $FRONTEND_DOCKER_URL

cd ..
docker build --no-cache --secret id=JUMPSELLER --secret id=PUBSUB --secret id=PASSWORD --secret id=SALT -t $BACKEND_DOCKER_URL .
docker push $BACKEND_DOCKER_URL

cd infra
tofu apply -var "frontend_image=$FRONTEND_DOCKER_URL" -var "backend_image=$BACKEND_DOCKER_URL"
