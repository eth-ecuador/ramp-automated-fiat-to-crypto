# Use the official Node.js image as the base
FROM node:18

# Set the working directory inside the container
WORKDIR /app

# Copy only package.json first for faster Docker caching
COPY package.json ./

# Install dependencies
RUN npm install

# Copy the rest of your app's code
COPY . .

# Expose port 3001 inside the container
EXPOSE 3001

# Start the Node.js app
CMD ["node", "index.js"]
