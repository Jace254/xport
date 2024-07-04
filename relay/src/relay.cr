require "socket"
require "json"

class Client
  property type : String
  property socket : TCPSocket

  def initialize(type : String, socket : TCPSocket)
    @type = type
    @socket = socket
  end
end

server = TCPServer.new("0.0.0.0", 8080)
puts "Server listening on port 8080"

clients = [] of Client
agents = [] of Client

# Handle incoming messages from agent and client
def handle_client(client : Client, agents : Array(Client), clients : Array(Client))
  puts "Init handler for #{client.type}"
  while message = client.socket.gets(chomp: false)
    if message 
      if client.type == "agent"
        puts "Sending message to client"
        clients.each { |c| c.socket.puts message }
      elsif client.type == "ui"
        message = message.chomp
        puts "Sending message to agent"
        agents.each { |a| a.socket.puts message }
      end
    end
  end

  if client.type == "agent"
    agents.delete(client)
  else
    clients.delete(client)
  end
rescue exception
  puts "Exception encountered: #{exception}"
ensure
  client.socket.close
  puts "#{client.type.capitalize} disconnected"
end

while socket = server.accept
  # Read the initial client type message
  client_type = socket.gets
  if client_type
    client_type = client_type.chomp
    if client_type == "agent" || client_type == "ui"
      client = Client.new(client_type, socket)
      if client_type == "agent"
        agents << client
      else
        clients << client
      end
      puts "#{client_type.capitalize} client connected"
      spawn handle_client(client, agents, clients)
    else
      socket.puts "Invalid client type"
      socket.close
    end
  else
    socket.close
  end
end