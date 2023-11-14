<template>
    <table>
        <thead>
            <tr>

                <th>MAC Source</th>
                <th>MAC Destination</th>
                <th>L2 Interface</th>
                <th>L3 Proto</th>
                <th>IP Source</th>
                <th>IP Destination</th>
                <th>Port Destination</th>
                <th>Port Source</th>
                <th>L4 Proto</th>
            </tr>
        </thead>
            <tbody>
                <tr v-for="(packet, index) in packets" :key="index">

                    <td>{{ packet.macSource }}</td>
                    <td>{{ packet.macDestination }}</td>
                    <td>{{ packet.l2Interface }}</td>
                    <td>{{ packet.l3Proto }}</td>
                    <td>{{ packet.ipSource }}</td>
                    <td>{{ packet.ipDestination }}</td>
                    <td>{{ packet.portDestination }}</td>
                    <td>{{ packet.portSource }}</td>
                    <td>{{ packet.l4Proto }}</td>
                </tr>
        </tbody>
    </table>
</template>

<script>
export default {
  name: 'NetworkPacketsTable',
  data() {
    return {
      // Exemple de données fictives
      packets: [
        // Ajoutez ici vos données de paquets
        { layer3: 'IPv4', 
        macSource: '00:00:00:00:00:00', macDestination: '00:00:00:00:00:00', 
        l2Interface: 'lo', l3Proto: 'IPv4', 
        ipSource: '127.0.0.1', ipDestination: '127.0.0.1', 
        portDestination: 52, portSource: 17664, 
        l4Proto: 'TCP' },
        // Plus de lignes ici si nécessaire
      ]
    };
  }
};
</script>

<style>
/* Styles pour le tableau */
table {
  width: 100%;
  border-collapse: collapse;
  background-color: #0a0a0a;
}
th, td {
  border: 1px solid rgb(184, 178, 178);
  padding: 8px;
  text-align: left;
}
thead {
  background-color: #0a0a0a;
}
</style>