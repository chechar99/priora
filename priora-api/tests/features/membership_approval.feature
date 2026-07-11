# language: es
Característica: Aprobación de usuarios por espacio
  Para evitar cuentas múltiples en temas serios
  Como administrador de un espacio
  Quiero exigir autorización antes de que priorizaciones y comentarios tengan efecto

  Antecedentes:
    Dado un espacio "barrio-test" con datos mínimos
    Y un administrador de plataforma autenticado
    Y un usuario regular autenticado con perfil completo

  Escenario: Sin aprobación requerida la participación es libre
    Dado que el espacio no requiere aprobación de usuarios
    Cuando el usuario regular consulta su membresía
    Entonces puede comentar en el espacio
    Y su priorización cuenta en el ranking
    Cuando el usuario regular publica un comentario
    Entonces la respuesta es exitosa
    Cuando el usuario regular guarda su priorización
    Entonces la respuesta es exitosa
    Y el score de la propuesta refleja su priorización

  Escenario: Con aprobación requerida no puede comentar hasta ser autorizado
    Dado que el espacio requiere aprobación de usuarios
    Cuando el usuario regular consulta su membresía
    Entonces no puede comentar en el espacio
    Y su priorización no cuenta en el ranking
    Cuando el usuario regular intenta publicar un comentario
    Entonces la respuesta es prohibida

  Escenario: Solicitud, priorización sin efecto y aprobación
    Dado que el espacio requiere aprobación de usuarios
    Cuando el usuario regular solicita autorización
    Entonces su membresía queda en estado "pending"
    Cuando el usuario regular guarda su priorización
    Entonces la respuesta es exitosa
    Y el score de la propuesta no refleja su priorización
    Cuando el administrador aprueba al usuario regular
    Entonces su membresía queda en estado "active"
    Y puede comentar en el espacio
    Y su priorización cuenta en el ranking
    Y el score de la propuesta refleja su priorización

  Escenario: Rechazo de solicitud
    Dado que el espacio requiere aprobación de usuarios
    Cuando el usuario regular solicita autorización
    Y el administrador rechaza al usuario regular
    Entonces su membresía queda en estado "rejected"
    Y no puede comentar en el espacio
    Cuando el usuario regular intenta publicar un comentario
    Entonces la respuesta es prohibida

  Escenario: Admin de espacio puede aprobar solicitudes
    Dado que el espacio requiere aprobación de usuarios
    Y un admin de espacio autenticado
    Cuando el usuario regular solicita autorización
    Y el admin de espacio aprueba al usuario regular
    Entonces su membresía queda en estado "active"
    Y puede comentar en el espacio
