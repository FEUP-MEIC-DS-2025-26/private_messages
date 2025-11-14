# mips_product_page/Dockerfile
FROM node:25-alpine3.21

# Set workdir
WORKDIR /app

# Copy pnpm files
COPY package.json package-lock.json ./

# Install dependencies
RUN npm install

# Copy source code
COPY . .

# Rsbuild/Vite dev servers need this to be accessible
ENV HOST=0.0.0.0

# Start the development server
# TODO: Figure out how to use production image.
CMD ["npm", "run", "dev"]