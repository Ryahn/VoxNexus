FROM node:20-slim

# Install required packages
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    clamav \
    clamav-daemon \
    clamav-freshclam \
    curl \
    cron \
    wget \
    tar \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Update ClamAV virus definitions
RUN freshclam --datadir=/var/lib/clamav

# Configure ClamAV daemon
RUN mkdir -p /run/clamav && \
    chown clamav:clamav /run/clamav && \
    mkdir -p /var/lib/clamav && \
    chown clamav:clamav /var/lib/clamav

# Install maldet
RUN wget http://www.rfxn.com/downloads/maldetect-current.tar.gz && \
    tar -xzf maldetect-current.tar.gz && \
    cd maldetect-* && \
    yes | ./install.sh

# Copy our custom maldet configuration
COPY conf.maldet /usr/local/maldetect/conf.maldet
RUN chmod 644 /usr/local/maldetect/conf.maldet

# Create directory for scan results
RUN mkdir -p /var/run/clamav && \
    chown clamav:clamav /var/run/clamav

# Copy application files
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .

# Create log directory for cron jobs
RUN mkdir -p /var/log && \
    touch /var/log/avscanner-cleanup.log && \
    chmod 666 /var/log/avscanner-cleanup.log

# Install cron job
RUN echo "0 0 * * * /usr/local/bin/node /app/src/cleanup_old_files.js >> /var/log/avscanner-cleanup.log 2>&1" | crontab -

# Find clamdscan location and create symlink
RUN which clamdscan || ln -s /usr/bin/clamdscan /usr/local/bin/clamdscan

# Create startup script
RUN printf '#!/bin/sh\n\
export PATH="/usr/local/bin:/usr/bin:/usr/sbin:$PATH"\n\
service clamav-daemon start\n\
service cron start\n\
cd /app\n\
exec /usr/local/bin/node src/index.js\n' > /start.sh && \
    chmod +x /start.sh

# Expose port
EXPOSE 3000

# Start the application
CMD ["/start.sh"]
