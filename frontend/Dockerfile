FROM node:20-alpine AS frontend
WORKDIR /app/frontend
COPY next.config.ts package.json package-lock.json postcss.config.mjs tsconfig.json ./
COPY app ./app
COPY public ./public
RUN npm install
RUN npx next build

EXPOSE 3000
CMD ["npm", "run", "start"]
