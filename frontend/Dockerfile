# mips_product_page/Dockerfile
FROM node:25-alpine3.21

# Set workdir
WORKDIR /app

# Copy pnpm files
COPY package.json package-lock.json ./

# Install dependencies
RUN npm install -g pnpm && pnpm install

# Copy source code
COPY . .

# Rsbuild/Vite dev servers need this to be accessible
ENV HOST=0.0.0.0

EXPOSE 3001

ENV PUBLIC_BACKEND_URL="https://backend-private-messages-azzawq6caq-no.a.run.app"

RUN pnpm build

CMD ["pnpm", "preview"]
