require "socket"
require "json"
require "log"

Log.setup :debug

class Client
  property type : String
  property socket : TCPSocket

  def initialize(type : String, socket : TCPSocket)
    @type = type
    @socket = socket
  end
end

class SafeClientList
  def initialize
    @clients = [] of Client
    @mutex = Mutex.new
  end

  def add(client : Client)
    @mutex.synchronize { @clients << client }
  end

  def delete(client : Client)
    @mutex.synchronize { @clients.delete(client) }
  end

  def each
    @mutex.synchronize { @clients.each { |client| yield client } }
  end

  def size
    @mutex.synchronize { @clients.size }
  end

  def empty?
    @mutex.synchronize { @clients.empty? }
  end
end

server = TCPServer.new("0.0.0.0", 8080)
Log.info { "Server listening on port 8080" }

agents = SafeClientList.new
clients = SafeClientList.new

def handle_client(client : Client, agents : SafeClientList, clients : SafeClientList)
  Log.info { "Init handler for #{client.type}" }
  loop do
    begin
      message = client.socket.gets(chomp: false)
      break if message.nil? # Client disconnected
      
      if client.type == "agent"
        if clients.empty?
          Log.info { "No clients connected. Closing agent connection." }
          break
        end
        # Log.debug { "Sending message to clients (#{clients.size})" }
        clients_to_remove = [] of Client
        clients.each do |c| 
          begin
            c.socket.puts message
          rescue ex : IO::Error
            Log.error(exception: ex) { "Error sending to client" }
            if ex.message.try &.includes?("An existing connection was forcibly closed by the remote host")
              clients_to_remove << c
            end
          rescue ex
            Log.error(exception: ex) { "Unexpected error sending to client" }
          end
        end
        
        # Remove disconnected clients
        clients_to_remove.each do |c|
          clients.delete(c)
          Log.info { "Client disconnected due to connection error" }
        end
        
        # If all clients disconnected, close the agent connection
        if clients.empty?
          Log.info { "All clients disconnected. Closing agent connection." }
          break
        end
      elsif client.type == "ui"
        Log.debug { "Sending message to agents (#{agents.size})" }
        agents.each do |a| 
          begin
            a.socket.puts message.chomp
          rescue ex
            Log.error(exception: ex) { "Error sending to agent" }
          end
        end
      end
    rescue ex : IO::Error
      Log.error(exception: ex) { "Connection error for #{client.type}" }
      break
    rescue ex
      Log.error(exception: ex) { "Unexpected error for #{client.type}" }
      break
    end
  end
rescue ex
  Log.error(exception: ex) { "Exception in handle_client for #{client.type}" }
ensure
  if client.type == "agent"
    agents.delete(client)
  else
    clients.delete(client)
  end
  client.socket.close rescue nil
  Log.info { "#{client.type.capitalize} disconnected" }
end

while socket = server.accept?
  client_type = socket.gets
  if client_type
    client_type = client_type.chomp
    if client_type == "agent" || client_type == "ui"
      client = Client.new(client_type, socket)
      if client_type == "agent"
        agents.add(client)
      else
        clients.add(client)
      end
      Log.info { "#{client_type.capitalize} client connected" }
      spawn handle_client(client, agents, clients)
    else
      socket.puts "Invalid client type"
      socket.close
    end
  else
    socket.close
  end
end