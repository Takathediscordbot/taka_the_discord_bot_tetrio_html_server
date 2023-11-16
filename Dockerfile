FROM debian:buster
WORKDIR /taka_the_discord_bot_tetrio_html_server
COPY --from=taka_the_discord_bot_dependencies /app/build/taka_the_discord_bot_tetrio_html_server .
COPY --from=taka_the_discord_bot_dependencies /app/taka_the_discord_bot_tetrio_html_server/.env ./.env
RUN  apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates
CMD ["/taka_the_discord_bot_tetrio_html_server/taka_the_discord_bot_tetrio_html_server"]