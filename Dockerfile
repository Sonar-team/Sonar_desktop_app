# Utiliser une image de base adaptée pour ARM
FROM rust:latest

# Installer les dépendances nécessaires
RUN apt-get update && \
    apt-get install -y \
    libgtk-3-dev \
    webkit2gtk-4.0 \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libpcap-dev

# Installer Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs

# Définir le répertoire de travail
WORKDIR /app

# Copier le fichier de dépendances et installer les dépendances
COPY package.json package-lock.json ./
RUN npm install

# Copier le reste du code
COPY . .

# Construire l'application
RUN npm run tauri build



