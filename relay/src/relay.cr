require "socket"
require "json"
require "log"
require "gc"

# TODO: Create separate functions for agents and clients
Log.setup :debug

record Client, type : String, socket : TCPSocket

class SafeClientList
  @clients : Array(Client)
  @mutex : Mutex

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

# class Message 
#   @message : String
#   @mutex : Mutex

#   def initialize
#     @message = ""
#     @mutex = Mutex.new
#   end

#   def set(message : String)
#     @mutex.synchronize { @message = message }
#   end

#   def get
#     @mutex.synchronize { @message }
#   end

#   def finalize
#     @mutex.synchronize { @message = "" }
#   end
# end

def handle_client(client : Client, agents : SafeClientList, clients : SafeClientList)
  Log.info { "Init handler for #{client.type}" }
  message = ""
  clients_to_remove = [] of Client

  loop do
    begin
      message = client.socket.gets(chomp: false) || break
      
      if client.type == "agent"
        if clients.empty?
          Log.info { "No clients connected. Closing agent connection." }
          break
        elsif
          # Log.debug { "Sending message to clients (#{clients.size})" }
          clients.each do |c|
            begin
              c.socket.puts(message)
            rescue ex : IO::Error
              Log.error(exception: ex) { "Error sending to client" }
              if ex.message.try &.includes?("An existing connection was forcibly closed by the remote host")
                clients_to_remove << c
              end
            rescue ex
              Log.error(exception: ex) { "Unexpected error sending to client" }
            end
          end
        end
        
        # Remove disconnected clients
        clients_to_remove.each do |c|
          clients.delete(c)
          Log.info { "Client disconnected due to connection error" }
        end
        clients_to_remove.clear
        
        # If all clients disconnected, close the agent connection
        break if clients.empty?
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
      message = nil
      client.socket.flush
    rescue ex : IO::Error
      Log.error(exception: ex) { "Connection error for #{client.type}: #{ex.message}" }
      break
    rescue ex
      Log.error(exception: ex) { "Unexpected error for #{client.type}: #{ex.message}" }
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

server = TCPServer.new("0.0.0.0", 8080)
Log.info { "Server listening on port 8080" }

agents  = SafeClientList.new
clients = SafeClientList.new

while socket = server.accept?
  client_type = socket.gets(chomp: true) # "agent\njoash"
  if client_type && {"agent", "ui"}.includes?(client_type) 
    client = Client.new(client_type, socket)
    (client_type == "agent" ? agents : clients).add(client)
    Log.info { "#{client_type.capitalize} client connected" }
    spawn handle_client(client, agents, clients)
  else
    socket.puts("Invalid client type")
    socket.close
  end
end